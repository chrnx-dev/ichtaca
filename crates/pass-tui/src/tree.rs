//! Flatten the `EntryNode` tree into the visible rows the UI navigates, given
//! the set of expanded directory paths. Pure: no terminal, no state mutation.

use std::collections::BTreeSet;

use passcore::EntryNode;

/// One visible line in the tree panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatRow {
    pub name: String,
    /// Full store path for a leaf (`web/github.com`); `None` for a directory.
    pub path: Option<String>,
    /// Directory path used as the expand/collapse key (`web`, `web/aws`).
    pub dir_key: Option<String>,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
}

/// Directions the selection can move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nav {
    Up,
    Down,
    Top,
    Bottom,
}

/// Produce the visible rows in display order.
pub fn flatten(roots: &[EntryNode], expanded: &BTreeSet<String>) -> Vec<FlatRow> {
    let mut out = Vec::new();
    walk(roots, expanded, 0, "", &mut out);
    out
}

fn walk(
    nodes: &[EntryNode],
    expanded: &BTreeSet<String>,
    depth: usize,
    prefix: &str,
    out: &mut Vec<FlatRow>,
) {
    for node in nodes {
        let is_dir = !node.children.is_empty() || node.path.is_none();
        let dir_path = if prefix.is_empty() {
            node.name.clone()
        } else {
            format!("{prefix}/{}", node.name)
        };
        if is_dir {
            let is_expanded = expanded.contains(&dir_path);
            out.push(FlatRow {
                name: node.name.clone(),
                path: None,
                dir_key: Some(dir_path.clone()),
                depth,
                is_dir: true,
                expanded: is_expanded,
            });
            if is_expanded {
                walk(&node.children, expanded, depth + 1, &dir_path, out);
            }
        } else {
            out.push(FlatRow {
                name: node.name.clone(),
                path: node.path.clone(),
                dir_key: None,
                depth,
                is_dir: false,
                expanded: false,
            });
        }
    }
}

/// Compute the new selection index after a navigation, clamped to `[0, len)`.
pub fn move_selection(current: usize, len: usize, nav: Nav) -> usize {
    if len == 0 {
        return 0;
    }
    let last = len - 1;
    match nav {
        Nav::Up => current.saturating_sub(1),
        Nav::Down => (current + 1).min(last),
        Nav::Top => 0,
        Nav::Bottom => last,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use passcore::EntryNode;
    use std::collections::BTreeSet;

    fn sample() -> Vec<EntryNode> {
        EntryNode::from_paths(&[
            "email/work".to_string(),
            "web/github.com".to_string(),
            "web/gitlab.com".to_string(),
        ])
    }

    #[test]
    fn collapsed_shows_only_top_level_dirs() {
        let roots = sample();
        let expanded = BTreeSet::new();
        let rows = flatten(&roots, &expanded);
        let names: Vec<_> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["email", "web"]);
        assert!(rows.iter().all(|r| r.is_dir));
    }

    #[test]
    fn expanding_a_dir_reveals_its_children_indented() {
        let roots = sample();
        let mut expanded = BTreeSet::new();
        expanded.insert("web".to_string());
        let rows = flatten(&roots, &expanded);
        let view: Vec<_> = rows
            .iter()
            .map(|r| (r.name.as_str(), r.depth, r.is_dir))
            .collect();
        assert_eq!(
            view,
            vec![
                ("email", 0, true),
                ("web", 0, true),
                ("github.com", 1, false),
                ("gitlab.com", 1, false),
            ]
        );
    }

    #[test]
    fn leaf_rows_carry_their_full_path() {
        let roots = sample();
        let mut expanded = BTreeSet::new();
        expanded.insert("web".to_string());
        let rows = flatten(&roots, &expanded);
        let gh = rows.iter().find(|r| r.name == "github.com").unwrap();
        assert_eq!(gh.path.as_deref(), Some("web/github.com"));
    }

    #[test]
    fn move_down_and_up_clamp_at_bounds() {
        assert_eq!(move_selection(0, 3, Nav::Down), 1);
        assert_eq!(move_selection(2, 3, Nav::Down), 2); // clamp at last
        assert_eq!(move_selection(0, 3, Nav::Up), 0); // clamp at first
        assert_eq!(move_selection(2, 3, Nav::Top), 0);
        assert_eq!(move_selection(0, 3, Nav::Bottom), 2);
    }

    #[test]
    fn move_on_empty_tree_is_zero() {
        assert_eq!(move_selection(0, 0, Nav::Down), 0);
        assert_eq!(move_selection(0, 0, Nav::Bottom), 0);
    }
}
