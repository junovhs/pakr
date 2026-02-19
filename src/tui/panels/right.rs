use crate::{scanner, types::AppState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::{collections::HashSet, path::PathBuf};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let mut iter = chunks.iter().copied();
    if let (Some(tree_area), Some(status_area)) = (iter.next(), iter.next()) {
        let selected: HashSet<PathBuf> = state.selected_paths().into_iter().collect();
        render_tree(frame, tree_area, state, &selected);
        render_status(frame, status_area, state, &selected);
    }
}

fn render_tree(frame: &mut Frame, area: Rect, state: &AppState, selected: &HashSet<PathBuf>) {
    let flat = scanner::flatten_visible(&state.tree);
    let items: Vec<ListItem> = flat
        .iter()
        .map(|item| make_item(item, selected, state))
        .collect();

    let mut ls = ListState::default();
    ls.select(Some(state.tree_cursor));

    let list = List::new(items)
        .block(Block::default().title("PREVIEW TREE").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_stateful_widget(list, area, &mut ls);
}

fn make_item(
    item: &scanner::FlatItem,
    selected: &HashSet<PathBuf>,
    state: &AppState,
) -> ListItem<'static> {
    let indent = "  ".repeat(item.depth);
    let is_nixed = state.nix.contains(&item.path);
    let is_selected = !is_nixed && selected.contains(&item.path);

    let status = if is_nixed {
        "✗"
    } else if is_selected {
        "✓"
    } else {
        " "
    };
    let icon = if item.is_dir {
        if item.expanded {
            "▼ "
        } else {
            "▶ "
        }
    } else {
        "  "
    };
    let label = format!("{indent}{status} {icon}{}", item.name);

    let color = if is_nixed {
        Color::DarkGray
    } else if is_selected {
        Color::White
    } else {
        Color::DarkGray
    };

    ListItem::new(Line::from(Span::styled(label, Style::default().fg(color))))
}

fn render_status(frame: &mut Frame, area: Rect, state: &AppState, selected: &HashSet<PathBuf>) {
    let n = selected.len();
    let bytes: u64 = selected
        .iter()
        .filter_map(|p| state.file_sizes.get(p))
        .sum();
    let tokens = bytes / 4;

    let summary_line = format!(" {n} files  {}  ~{}k tok", fmt_bytes(bytes), tokens / 1000);
    let hint = &state.status;
    let width = usize::from(area.width);

    let sep = "  |  ";
    let available = width.saturating_sub(summary_line.len() + sep.len());
    let hint_trimmed = if hint.len() > available {
        &hint[..available]
    } else {
        hint.as_str()
    };

    let text = format!("{summary_line}{sep}{hint_trimmed}");
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

fn fmt_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    if bytes < KB {
        format!("{bytes} B")
    } else if bytes < MB {
        format!("{} KB", bytes / KB)
    } else {
        format!("{} MB", bytes / MB)
    }
}
