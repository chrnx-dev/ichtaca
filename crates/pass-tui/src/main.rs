//! pass-tui — Ichtaca TUI frontend (tui-realm edition).
//!
//! Phase 1: blank themed window (Header + StatusBar) that quits on `q`/Ctrl-C.

mod components;
mod domain;
mod id;
mod model;
mod msg;
mod theme;

use std::time::Duration;

use tuirealm::application::{Application, PollStrategy};
use tuirealm::event::{Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::listener::EventListenerCfg;
use tuirealm::subscription::{EventClause, Sub, SubClause};
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalAdapter};

use id::Id;
use model::Model;
use msg::Msg;

fn main() {
    if let Err(e) = run() {
        eprintln!("pass-tui: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Load config; fall back to defaults on error.
    let config = passcore::Config::load().unwrap_or_default();

    // Build the password store; fall back to a fake store on failure.
    let store: Box<dyn passcore::PasswordStore + Send> =
        match passcore::PassCliStore::detect(config.store_dir.clone()) {
            Ok(s) => Box::new(s),
            Err(_) => Box::new(passcore::FakeStore::new()),
        };

    // Initialise the terminal (crossterm).
    // CrosstermTerminalAdapter::new() installs the panic hook automatically,
    // and its Drop impl restores the terminal — no manual teardown needed.
    let mut terminal = CrosstermTerminalAdapter::new()?;
    terminal.enable_raw_mode()?;
    terminal.enter_alternate_screen()?;

    // Build the event listener: crossterm keyboard input + 250 ms tick.
    let listener_cfg = EventListenerCfg::<NoUserEvent>::default()
        .crossterm_input_listener(Duration::from_millis(20), 3)
        .tick_interval(Duration::from_millis(250));

    // Initialise tui-realm application.
    let app: Application<Id, Msg, NoUserEvent> = Application::init(listener_cfg);

    // Build the model and mount Phase-1 components.
    let mut model = Model {
        app,
        quit: false,
        redraw: true,
        store,
        config,
    };
    model.mount_phase1();

    // Subscribe StatusBar to global quit keys so they fire even when no
    // component has explicit focus.
    model
        .app
        .subscribe(
            &Id::StatusBar,
            Sub::new(
                EventClause::Keyboard(KeyEvent::new(Key::Char('q'), KeyModifiers::NONE)),
                SubClause::Always,
            ),
        )
        .ok(); // already subscribed is fine

    model
        .app
        .subscribe(
            &Id::StatusBar,
            Sub::new(
                EventClause::Keyboard(KeyEvent::new(Key::Esc, KeyModifiers::NONE)),
                SubClause::Always,
            ),
        )
        .ok();

    model
        .app
        .subscribe(
            &Id::StatusBar,
            Sub::new(
                EventClause::Keyboard(KeyEvent::new(Key::Char('c'), KeyModifiers::CONTROL)),
                SubClause::Always,
            ),
        )
        .ok();

    // ── Main loop ────────────────────────────────────────────────────────────
    loop {
        // Draw if needed.
        if model.redraw {
            model.view(&mut terminal);
            model.redraw = false;
        }

        // Poll for events; collect messages.
        match model
            .app
            .tick(PollStrategy::Once(Duration::from_millis(20)))
        {
            Err(_) => {
                // Listener died — exit gracefully.
                model.quit = true;
            }
            Ok(messages) => {
                if !messages.is_empty() {
                    model.redraw = true;
                }
                for msg in messages {
                    let mut next = Some(msg);
                    while let Some(m) = next {
                        next = model.update(Some(m));
                    }
                }
            }
        }

        if model.quit {
            break;
        }
    }

    // terminal is restored automatically by CrosstermTerminalAdapter's Drop.
    Ok(())
}
