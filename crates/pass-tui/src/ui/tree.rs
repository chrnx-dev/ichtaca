//! Left panel: the entry tree as a selectable list.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::state::AppState;
use crate::theme;
use crate::tree::flatten;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let rows = flatten(&state.roots, &state.expanded);
    let items: Vec<ListItem> = rows
        .iter()
        .map(|r| {
            let indent = "  ".repeat(r.depth);
            let (marker, marker_style) = if r.is_dir {
                if r.expanded {
                    ("▾ ", theme::key())
                } else {
                    ("▸ ", theme::key())
                }
            } else {
                ("  ", Style::default())
            };
            // Directory entries and leaves share the same text colour; the
            // selected row is styled via `highlight_style` on the List widget.
            let line = Line::from(vec![
                Span::styled(indent, Style::default()),
                Span::styled(marker, marker_style),
                Span::styled(r.name.clone(), theme::text()),
            ]);
            ListItem::new(line)
        })
        .collect();

    let mut list_state = ListState::default();
    if !rows.is_empty() {
        list_state.select(Some(state.selected.min(rows.len() - 1)));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(ratatui::text::Span::styled("Entries", theme::title()));

    let list = List::new(items)
        .block(block)
        .highlight_style(theme::selection());

    frame.render_stateful_widget(list, area, &mut list_state);
}
