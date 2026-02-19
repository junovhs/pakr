use crate::types::{AppState, Focus};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let sel_count = state.selected_paths().len();
    let sel_bytes = state.total_bytes();
    let sel_tokens = state.total_tokens();

    let has_subs = !state.subsystems.is_empty();
    let summary_h = calc_summary_h(state.exclude.items().len(), state.manual_includes.len());
    let chunks = split(area, has_subs, summary_h);

    let mut idx = 0usize;
    if let Some(&r) = chunks.get(idx) {
        state.cat_area = r;
        render_categories(frame, r, state);
        idx += 1;
    }
    if has_subs {
        if let Some(&r) = chunks.get(idx) {
            state.sub_area = r;
            render_subsystems(frame, r, state);
            idx += 1;
        }
    }
    if let Some(&r) = chunks.get(idx) {
        render_summary(frame, r, state, sel_count, sel_bytes, sel_tokens);
    }
}

fn calc_summary_h(excl_count: usize, manual_count: usize) -> u16 {
    let excl_rows = u16::try_from(excl_count.min(5)).unwrap_or(5);
    let manual_rows = u16::try_from(manual_count.min(3)).unwrap_or(3);
    10 + excl_rows + manual_rows
}

fn split(area: Rect, has_subs: bool, summary_h: u16) -> Vec<Rect> {
    let constraints = if has_subs {
        vec![
            Constraint::Min(6),
            Constraint::Min(4),
            Constraint::Length(summary_h),
        ]
    } else {
        vec![Constraint::Min(6), Constraint::Length(summary_h)]
    };
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area)
        .to_vec()
}

fn render_categories(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let focused = state.focus == Focus::Categories;
    let mut items: Vec<ListItem> = Vec::new();

    // Row 0: collapse/expand toggle
    let collapse_label = if state.all_collapsed {
        "  ▼ EXPAND ALL"
    } else {
        "  ▶ COLLAPSE ALL"
    };
    items.push(ListItem::new(Line::from(Span::styled(
        collapse_label,
        Style::default().fg(Color::DarkGray),
    ))));

    // Row 1: select/deselect all toggle
    let select_label = if state.are_all_selected() {
        "  ☑ DESELECT ALL"
    } else {
        "  ☐ SELECT ALL"
    };
    items.push(ListItem::new(Line::from(Span::styled(
        select_label,
        Style::default().fg(Color::DarkGray),
    ))));

    // Optional gitignore row
    if state.has_gitignore {
        let gi_check = if state.respect_gitignore {
            "✓"
        } else {
            "✗"
        };
        items.push(ListItem::new(Line::from(Span::styled(
            format!(" [{gi_check}] 🚫 .gitignore"),
            Style::default().fg(Color::Yellow),
        ))));
    }

    for cat in &state.categories {
        let check = if cat.enabled { "✓" } else { "✗" };
        let tok = cat.token_estimate(&state.file_sizes) / 1000;
        items.push(ListItem::new(Line::from(format!(
            " [{check}] {}  ~{tok}k",
            cat.name()
        ))));
    }

    let cursor = state.cat_cursor;
    state.cat_list_state.select(Some(cursor));
    let list = List::new(items)
        .block(panel_block("INCLUDE", focused))
        .highlight_style(hl_style(focused));
    frame.render_stateful_widget(list, area, &mut state.cat_list_state);
}

fn render_subsystems(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let focused = state.focus == Focus::Subsystems;
    let items: Vec<ListItem> = state
        .subsystems
        .iter()
        .map(|sub| {
            let check = if sub.enabled { "✓" } else { "✗" };
            let tok = sub.token_estimate(&state.file_sizes) / 1000;
            ListItem::new(Line::from(format!(" [{check}] {}  ~{tok}k", sub.name)))
        })
        .collect();

    let cursor = state.sub_cursor;
    state.sub_list_state.select(Some(cursor));
    let list = List::new(items)
        .block(panel_block("SUBSYSTEMS", focused))
        .highlight_style(hl_style(focused));
    frame.render_stateful_widget(list, area, &mut state.sub_list_state);
}

fn render_summary(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    sel_count: usize,
    sel_bytes: u64,
    sel_tokens: usize,
) {
    let focused_excl = state.focus == Focus::Exclude;
    let mut lines: Vec<Line> = vec![
        Line::from(format!(
            " {} files  {}  ~{}k tok",
            sel_count,
            fmt_bytes(sel_bytes),
            sel_tokens / 1000
        )),
        Line::default(),
        Line::from(Span::styled(" EXCLUDED  [e]focus  [d]remove", dim())),
    ];

    if state.exclude.is_empty() {
        lines.push(Line::from(Span::styled("  nothing excluded", dim())));
    } else {
        for (i, path) in state.exclude.items().iter().enumerate().take(5) {
            let style = if focused_excl && i == state.exclude_cursor {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default().fg(Color::Red)
            };
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            lines.push(Line::from(Span::styled(format!("  ✗ {name}"), style)));
        }
    }

    if !state.manual_includes.is_empty() {
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(" ADDED MANUALLY", dim())));
        for path in state.manual_includes.iter().take(3) {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            lines.push(Line::from(Span::styled(
                format!("  + {name}"),
                Style::default().fg(Color::Green),
            )));
        }
    }

    lines.push(Line::default());
    let gi_hint = if state.has_gitignore {
        "  [g]gitignore"
    } else {
        ""
    };
    lines.push(Line::from(Span::styled(
        format!(" [↑↓]nav  [spc]toggle  [tab]panel{gi_hint}"),
        dim(),
    )));
    lines.push(Line::from(Span::styled(
        " [n]excl  [a]add  [↵]copy  [f]save  [q]quit",
        dim(),
    )));

    let border_style = if focused_excl {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let widget = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .title("SUMMARY")
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    frame.render_widget(widget, area);
}

fn panel_block(title: &str, focused: bool) -> Block<'_> {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
}

fn hl_style(focused: bool) -> Style {
    if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::REVERSED)
    } else {
        Style::default().add_modifier(Modifier::REVERSED)
    }
}

fn dim() -> Style {
    Style::default().fg(Color::DarkGray)
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
