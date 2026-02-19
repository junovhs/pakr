use crate::{
    scanner,
    types::{AppState, Focus},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::{collections::HashSet, path::PathBuf};

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
    if state.input_mode {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(area);
        let mut iter = chunks.iter().copied();
        if let (Some(tree_area), Some(input_area)) = (iter.next(), iter.next()) {
            state.tree_area = tree_area;
            render_tree(frame, tree_area, state);
            render_input_bar(frame, input_area, state);
        }
    } else {
        state.tree_area = area;
        render_tree(frame, area, state);
    }
}

fn render_tree(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let selected: HashSet<PathBuf> = state.selected_paths().into_iter().collect();
    let flat = scanner::flatten_visible(&state.tree);
    let hover = state.hover_path.clone();
    let focused = state.focus == Focus::Tree;

    let items: Vec<ListItem> = flat
        .iter()
        .map(|item| {
            let is_excluded = state.exclude.contains(&item.path);
            let is_selected = !is_excluded && selected.contains(&item.path);
            let is_hovered = hover.as_deref() == Some(item.path.as_path());
            let tok = if item.is_dir {
                None
            } else {
                state
                    .file_sizes
                    .get(&item.path)
                    .map(|&s| usize::try_from(s / 3).unwrap_or(usize::MAX / 3))
            };
            make_item(item, is_selected, is_excluded, is_hovered, tok)
        })
        .collect();

    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let cursor = state.tree_cursor();
    state.tree_list_state.select(Some(cursor));
    let list = List::new(items)
        .block(
            Block::default()
                .title("PREVIEW TREE")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_stateful_widget(list, area, &mut state.tree_list_state);
}

fn make_item(
    item: &scanner::FlatItem,
    is_selected: bool,
    is_excluded: bool,
    is_hovered: bool,
    tok: Option<usize>,
) -> ListItem<'static> {
    let indent = "  ".repeat(item.depth);
    let status = if is_excluded {
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
    let tok_str = tok.map_or_else(String::new, |t| {
        if t >= 1000 {
            format!("  ~{}k", t / 1000)
        } else {
            format!("  ~{t}")
        }
    });
    let label = format!("{indent}{status} {icon}{}{tok_str}", item.name);

    let color = if is_excluded {
        Color::DarkGray
    } else if is_hovered {
        Color::Cyan
    } else if is_selected {
        Color::White
    } else {
        Color::DarkGray
    };

    ListItem::new(Line::from(Span::styled(label, Style::default().fg(color))))
}

fn render_input_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let text = format!(" {}_", state.input_buffer);
    let widget = Paragraph::new(text)
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .title("ADD FILE (relative path)  [↵]confirm  [esc]cancel")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );
    frame.render_widget(widget, area);
}
