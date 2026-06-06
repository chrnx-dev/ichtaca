//! pass-tui — a ratatui terminal frontend for `pass`, consuming `passcore`.
//!
//! This file is terminal glue only (untestable by design). Every decision —
//! input mapping, state transitions, rendering — lives in the pure sibling
//! modules, which are unit-tested.

mod action;
mod app;
mod form;
mod keymap;
mod otp;
mod search;
mod state;
mod tree;
mod ui;
mod update;

use std::io::{self, Stdout};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;
use state::{Mode, NoticeKind};

type Tui = Terminal<CrosstermBackend<Stdout>>;

const TICK: Duration = Duration::from_millis(250);

fn main() {
    if let Err(e) = run() {
        eprintln!("pass-tui: {e}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    // Load config; on parse error, fall back to defaults and remember the error
    // so we can surface it to the user as a notification after the app is built.
    let (config, config_err) = match passcore::Config::load() {
        Ok(c) => (c, None),
        Err(e) => (
            passcore::Config::default(),
            Some(format!("config ignored (parse error): {e}")),
        ),
    };

    // Build the store; on failure, show the help screen instead of crashing.
    let mut app = match passcore::PassCliStore::detect(config.store_dir.clone()) {
        Ok(store) => App::new(Box::new(store), config),
        Err(e) => {
            let mut a = App::new(Box::new(passcore::FakeStore::new()), config);
            a.state.mode = Mode::Help;
            a.state.notify(e.to_string(), NoticeKind::Error);
            a
        }
    };

    // Surface config parse errors as a status bar notification.
    if let Some(msg) = config_err {
        app.state.notify(msg, NoticeKind::Error);
    }

    let mut terminal = setup_terminal()?;
    install_panic_hook();
    let res = event_loop(&mut terminal, &mut app);
    teardown_terminal(&mut terminal)?;
    res
}

fn setup_terminal() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn teardown_terminal(terminal: &mut Tui) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

/// Restore the terminal on panic so the user is not left in a broken state.
fn install_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        hook(info);
    }));
}

fn event_loop(terminal: &mut Tui, app: &mut App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        // Pass the current unix timestamp so the detail panel renders a live
        // OTP code + countdown; the code is recomputed from the clock each frame.
        terminal.draw(|f| ui::render(f, &app.state, now_unix()))?;

        let timeout = TICK.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    handle_key(terminal, app, key)?;
                }
            }
        }
        if last_tick.elapsed() >= TICK {
            last_tick = Instant::now();
        }
        if app.state.should_quit {
            return Ok(());
        }
    }
}

fn handle_key(terminal: &mut Tui, app: &mut App, key: event::KeyEvent) -> io::Result<()> {
    let action = keymap::map(key, &app.state.mode, &app.config.keybindings);
    if let Some(effect) = update::update(&mut app.state, action) {
        if let action::SideEffect::RawEdit(path) = &effect {
            // Suspend the TUI, hand off to `$EDITOR` via the core, then restore.
            suspend_for_raw_edit(terminal, app, path)?;
        } else {
            app.perform(effect);
        }
    }
    Ok(())
}

/// Leave the alternate screen, run `pass edit` (which uses `$EDITOR`), restore.
fn suspend_for_raw_edit(terminal: &mut Tui, app: &mut App, path: &str) -> io::Result<()> {
    teardown_terminal(terminal)?;
    // The core owns the `pass edit` invocation (encrypted tmpfile handling).
    if let Err(e) = app.store.edit(path) {
        app.state.notify(e.to_string(), NoticeKind::Error);
    }
    *terminal = setup_terminal()?;
    app.reload_tree();
    // Reload the detail if it was the edited entry.
    if app.state.detail_path.as_deref() == Some(path) {
        app.perform(action::SideEffect::LoadDetail(path.to_string()));
    }
    Ok(())
}

/// Current unix time in seconds. Passed to `ui::render` on every frame so the
/// detail panel can display a live OTP code and countdown.
fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
