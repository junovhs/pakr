use crate::types::{AppState, Focus};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let show_subs = !state.subsystems.is_empty();
    let show_nix = !state.nix.is_empty();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(section_constraints(show_subs, show_nix))
        .split(area);

    let mut slots = chunks.iter().copied();
    if let Some(a) = slots.next() {
        render_categories(frame, a, state);
    }
    if show_subs {
        if let Some(a) = slots.next() {
            render_subsystems(frame, a, state);
        }
    }
    if show_nix {
        if let Some(a) = slots.next() {
            render_nix(frame, a, state);
        }
    }
}

fn section_constraints(has_subs: bool, has_nix: bool) -> Vec<Constraint> {
    match (has_subs, has_nix) {
        (true, true) => vec![
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Percentage(20),
        ],
        (true, false) | (false, true) => {
            vec![Constraint::Percentage(60), Constraint::Percentage(40)]
        }
        (false, false) => vec![Constraint::Percentage(100)],
    }
}

fn render_categories(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == Focus::Categories;
    let items: Vec<ListItem> = state
        .categories
        .iter()
        .map(|cat| {
            let check = if cat.enabled { "✓" } else { "✗" };
            let tok = cat.token_estimate(&state.file_sizes) / 1000;
            ListItem::new(Line::from(format!(
                " [{check}] {}  ~{tok}k tok",
                cat.name()
            )))
        })
        .collect();
    render_list(frame, area, "INCLUDE", items, state.cat_cursor, focused);
}

fn render_subsystems(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == Focus::Subsystems;
    let items: Vec<ListItem> = state
        .subsystems
        .iter()
        .map(|sub| {
            let check = if sub.enabled { "✓" } else { "✗" };
            let tok = sub.token_estimate(&state.file_sizes) / 1000;
            let n = sub.files.len();
            ListItem::new(Line::from(format!(
                " [{check}] {}  {n}f  ~{tok}k tok",
                sub.name
            )))
        })
        .collect();
    render_list(frame, area, "SUBSYSTEMS", items, state.sub_cursor, focused);
}

fn render_nix(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == Focus::Nix;
    let items: Vec<ListItem> = state
        .nix
        .items()
        .iter()
        .map(|p| {
            ListItem::new(Line::from(format!(" ✗ {}", p.display())))
                .style(Style::default().fg(Color::Red))
        })
        .collect();
    render_list(
        frame,
        area,
        "NIX  (d: remove)",
        items,
        state.nix_cursor,
        focused,
    );
}

fn render_list<'a>(
    frame: &mut Frame,
    area: Rect,
    title: &'a str,
    items: Vec<ListItem<'a>>,
    cursor: usize,
    focused: bool,
) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let hl = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::REVERSED)
    } else {
        Style::default().add_modifier(Modifier::REVERSED)
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);
    let mut ls = ListState::default();
    ls.select(Some(cursor));
    let list = List::new(items).block(block).highlight_style(hl);
    frame.render_stateful_widget(list, area, &mut ls);
}
