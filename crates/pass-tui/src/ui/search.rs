//! Search bar overlay: query line + filtered results.

use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem};
use ratatui::Frame;

use crate::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area);
    let mut items = vec![ListItem::new(format!("/{}", state.search.query))];
    items.extend(
        state
            .search
            .results
            .iter()
            .map(|p| ListItem::new(p.clone())),
    );
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Search"));
    frame.render_widget(list, area);
}
