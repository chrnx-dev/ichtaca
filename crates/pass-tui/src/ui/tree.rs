//! Left panel: the entry tree as a selectable list.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::state::AppState;
use crate::tree::flatten;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let rows = flatten(&state.roots, &state.expanded);
    let items: Vec<ListItem> = rows
        .iter()
        .map(|r| {
            let indent = "  ".repeat(r.depth);
            let marker = if r.is_dir {
                if r.expanded {
                    "▾ "
                } else {
                    "▸ "
                }
            } else {
                "  "
            };
            ListItem::new(format!("{indent}{marker}{}", r.name))
        })
        .collect();

    let mut list_state = ListState::default();
    if !rows.is_empty() {
        list_state.select(Some(state.selected.min(rows.len() - 1)));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Entries"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(list, area, &mut list_state);
}
