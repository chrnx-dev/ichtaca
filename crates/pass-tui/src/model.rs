//! Application model — owns the tui-realm `Application` and all domain state.
//!
//! `Model::new` mounts the Phase-1 components; later phases add more.
//! `Model::view` draws the three-row layout (Header / content / StatusBar).
//! `Model::update` processes messages from the event loop.

use tuirealm::application::Application;
use tuirealm::event::NoUserEvent;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::ratatui::Frame;
use tuirealm::terminal::TerminalAdapter;

use crate::components::{Header, StatusBar};
use crate::id::Id;
use crate::msg::Msg;
use crate::theme;

/// Central model for the Ichtaca TUI.
#[allow(dead_code)] // `store` and `config` are used from Phase 2 onwards
pub struct Model {
    /// The tui-realm application (view + subscriptions + event listener).
    pub app: Application<Id, Msg, NoUserEvent>,
    /// Set to `true` when the app should exit the main loop.
    pub quit: bool,
    /// Set to `true` when the terminal needs to be redrawn.
    pub redraw: bool,
    /// Password store backend.
    pub store: Box<dyn passcore::PasswordStore + Send>,
    /// User configuration.
    pub config: passcore::Config,
}

impl Model {
    /// Mount Phase-1 components (Header + StatusBar) into the application.
    ///
    /// Call this once after `Application::init`.
    pub fn mount_phase1(&mut self) {
        self.app
            .mount(Id::Header, Box::new(Header::default()), vec![])
            .expect("mount Header");

        self.app
            .mount(Id::StatusBar, Box::new(StatusBar::default()), vec![])
            .expect("mount StatusBar");

        // Give StatusBar initial focus so it receives keyboard events via the
        // global subscriptions registered in main (q / Ctrl-C).
        self.app.active(&Id::StatusBar).expect("activate StatusBar");
    }

    /// Draw the current frame.
    ///
    /// Layout (top → bottom):
    /// - Row 0 (1 line)  : Header
    /// - Row 1 (fill)    : empty middle area (tree + detail arrive in Phase 2)
    /// - Row 2 (1 line)  : StatusBar
    pub fn view<T: TerminalAdapter>(&mut self, terminal: &mut T) {
        let _ = terminal.draw(|f: &mut Frame| {
            Self::render_frame(&mut self.app, f);
        });
    }

    fn render_frame(app: &mut Application<Id, Msg, NoUserEvent>, f: &mut Frame) {
        let area = f.area();

        // Fill background
        let bg_block = tuirealm::ratatui::widgets::Block::default()
            .style(tuirealm::ratatui::style::Style::default().bg(theme::BG));
        f.render_widget(bg_block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Fill(1),   // content (empty for now)
                Constraint::Length(1), // StatusBar
            ])
            .split(area);

        app.view(&Id::Header, f, chunks[0]);
        // chunks[1] stays empty until Phase 2 mounts the Tree + Detail
        app.view(&Id::StatusBar, f, chunks[2]);
    }

    /// Process one message from the event loop.
    ///
    /// Returns `Some(next_msg)` to chain handling; `None` when done.
    pub fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        match msg {
            None => None,
            Some(Msg::None) => None,
            Some(Msg::Quit) => {
                self.quit = true;
                None
            }
            Some(Msg::Tick) => {
                self.redraw = true;
                None
            }
            // Phase 2–3 messages are received but not yet acted on.
            Some(_) => None,
        }
    }
}
