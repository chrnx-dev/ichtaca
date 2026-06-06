//! Search bar overlay: query line + filtered results.

use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState};
use ratatui::Frame;

use crate::state::AppState;
use crate::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(Clear, area);

    // The first item is the query line; results start at index 1.
    // We offset the cursor by 1 to account for the query row.
    let mut items = vec![ListItem::new(format!("/{}", state.search.query))];
    items.extend(
        state
            .search
            .results
            .iter()
            .map(|p| ListItem::new(p.clone())),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled("Search", theme::title()));

    let list = List::new(items)
        .block(block)
        .highlight_style(theme::selection())
        .highlight_symbol("> ");

    // Offset cursor by 1: index 0 is the query line, results start at 1.
    let mut list_state = ListState::default();
    if !state.search.results.is_empty() {
        list_state.select(Some(state.search.cursor + 1));
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use passcore::EntryNode;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn draw_search(state: &AppState) -> String {
        let backend = TestBackend::new(60, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                render(f, area, state);
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let mut out = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    fn search_state_with_results() -> AppState {
        let mut s = AppState::new();
        s.roots = EntryNode::from_paths(&[
            "web/github.com".to_string(),
            "web/gitlab.com".to_string(),
            "email/work".to_string(),
        ]);
        crate::update::update(&mut s, crate::action::Action::EnterSearch);
        // Type nothing so all results are shown.
        s.search.recompute(&[
            "web/github.com".to_string(),
            "web/gitlab.com".to_string(),
            "email/work".to_string(),
        ]);
        s
    }

    #[test]
    fn search_renders_query_line_and_results() {
        let s = search_state_with_results();
        let out = draw_search(&s);
        assert!(out.contains("Search"), "missing Search title");
        assert!(out.contains("web/github.com"), "missing first result");
    }

    #[test]
    fn search_cursor_zero_highlights_first_result() {
        let mut s = search_state_with_results();
        s.search.cursor = 0;
        let out = draw_search(&s);
        // The highlighted row should contain "> " prefix and the first result path.
        assert!(
            out.contains("> "),
            "expected '> ' highlight symbol, got:\n{out}"
        );
        // The first result (cursor=0) should appear highlighted.
        // We check the highlight prefix appears near the first result.
        let lines: Vec<&str> = out.lines().collect();
        let highlighted = lines
            .iter()
            .any(|l| l.contains("> ") && l.contains("web/github.com"));
        assert!(
            highlighted,
            "expected first result 'web/github.com' to be on a highlighted line (with '>'), got:\n{out}"
        );
    }

    #[test]
    fn search_cursor_moved_highlights_correct_result() {
        let mut s = search_state_with_results();
        // Move cursor to index 1 (second result: web/gitlab.com).
        s.search.cursor = 1;
        let out = draw_search(&s);
        let lines: Vec<&str> = out.lines().collect();
        let highlighted = lines
            .iter()
            .any(|l| l.contains("> ") && l.contains("web/gitlab.com"));
        assert!(
            highlighted,
            "expected 'web/gitlab.com' (cursor=1) to be highlighted, got:\n{out}"
        );
    }
}
