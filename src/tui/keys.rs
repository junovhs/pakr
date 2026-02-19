use crate::{
    output, packer, scanner,
    types::{AppState, Focus},
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use std::path::PathBuf;

pub fn handle_key(key: KeyEvent, state: &mut AppState) -> Result<bool> {
    if key.kind != KeyEventKind::Press {
        return Ok(false);
    }
    if state.input_mode {
        handle_input_key(key, state);
        return Ok(false);
    }
    match key.code {
        KeyCode::Char('q' | 'Q') => return Ok(true),
        KeyCode::Tab => cycle_focus(state),
        KeyCode::Esc => escape_focus(state),
        KeyCode::Up => move_cursor(state, false),
        KeyCode::Down => move_cursor(state, true),
        KeyCode::Char(' ') => toggle_current(state),
        KeyCode::Char('n') => exclude_at_cursor(state),
        KeyCode::Char('d') | KeyCode::Delete => unexclude_current(state),
        KeyCode::Char('e') => {
            state.focus = Focus::Exclude;
        }
        KeyCode::Char('g') => toggle_gitignore(state),
        KeyCode::Char('a') => start_input(state),
        KeyCode::Enter => do_clipboard(state)?,
        KeyCode::Char('f') => do_file(state)?,
        KeyCode::Left | KeyCode::Right => toggle_expand(state),
        _ => {}
    }
    state.clamp_cursors();
    Ok(false)
}

fn handle_input_key(key: KeyEvent, state: &mut AppState) {
    match key.code {
        KeyCode::Esc => {
            state.input_mode = false;
            state.input_buffer.clear();
        }
        KeyCode::Enter => {
            let path = PathBuf::from(state.input_buffer.trim());
            if !path.as_os_str().is_empty() && !state.manual_includes.contains(&path) {
                state.manual_includes.push(path);
            }
            state.input_mode = false;
            state.input_buffer.clear();
        }
        KeyCode::Backspace => {
            state.input_buffer.pop();
        }
        KeyCode::Char(c) => {
            state.input_buffer.push(c);
        }
        _ => {}
    }
}

fn start_input(state: &mut AppState) {
    state.input_mode = true;
    state.input_buffer.clear();
}

pub fn handle_mouse(mouse: MouseEvent, state: &mut AppState) -> Result<()> {
    match mouse.kind {
        MouseEventKind::Moved => handle_hover(mouse.column, mouse.row, state),
        MouseEventKind::Down(MouseButton::Left) => handle_click(mouse.column, mouse.row, state),
        _ => {}
    }
    Ok(())
}

fn handle_hover(col: u16, row: u16, state: &mut AppState) {
    if in_rect(state.tree_area, col, row) {
        let idx = tree_hit(row, state.tree_area, state.tree_list_state.offset());
        let flat = scanner::flatten_visible(&state.tree);
        state.hover_path = flat.get(idx).map(|i| i.path.clone());
    } else {
        state.hover_path = None;
    }
}

fn handle_click(col: u16, row: u16, state: &mut AppState) {
    if in_rect(state.tree_area, col, row) {
        click_tree(row, state);
    } else if in_rect(state.cat_area, col, row) {
        click_cat(row, state);
    } else if in_rect(state.sub_area, col, row) {
        click_sub(row, state);
    }
    let _ = col;
}

fn click_tree(row: u16, state: &mut AppState) {
    let idx = tree_hit(row, state.tree_area, state.tree_list_state.offset());
    let flat = scanner::flatten_visible(&state.tree);
    state.focus = Focus::Tree;
    if let Some(item) = flat.get(idx) {
        let path = item.path.clone();
        let is_dir = item.is_dir;
        state.set_tree_cursor(idx);
        if is_dir {
            scanner::toggle_node_expanded(&mut state.tree, &path);
        }
    }
}

fn click_cat(row: u16, state: &mut AppState) {
    let idx = row_offset(row, state.cat_area);
    state.focus = Focus::Categories;
    if idx < state.cat_list_len() {
        state.cat_cursor = idx;
    }
}

fn click_sub(row: u16, state: &mut AppState) {
    let idx = row_offset(row, state.sub_area);
    state.focus = Focus::Subsystems;
    if idx < state.subsystems.len() {
        state.sub_cursor = idx;
    }
}

fn tree_hit(row: u16, area: Rect, scroll: usize) -> usize {
    row_offset(row, area) + scroll
}

fn row_offset(row: u16, area: Rect) -> usize {
    row.saturating_sub(area.y + 1) as usize
}

fn in_rect(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x
        && col < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

fn cycle_focus(state: &mut AppState) {
    state.focus = match state.focus {
        Focus::Categories => {
            if state.subsystems.is_empty() {
                Focus::Tree
            } else {
                Focus::Subsystems
            }
        }
        Focus::Subsystems => Focus::Tree,
        Focus::Tree | Focus::Exclude => Focus::Categories,
    };
}

fn escape_focus(state: &mut AppState) {
    if state.focus == Focus::Exclude {
        state.focus = Focus::Categories;
    }
}

fn move_cursor(state: &mut AppState, down: bool) {
    match state.focus {
        Focus::Categories => {
            state.cat_cursor = step(state.cat_cursor, down, state.cat_list_len());
        }
        Focus::Subsystems => {
            state.sub_cursor = step(state.sub_cursor, down, state.subsystems.len());
        }
        Focus::Exclude => {
            let len = state.exclude.items().len();
            state.exclude_cursor = step(state.exclude_cursor, down, len);
        }
        Focus::Tree => {
            let len = scanner::flatten_visible(&state.tree).len();
            let next = step(state.tree_cursor(), down, len);
            state.set_tree_cursor(next);
        }
    }
}

fn step(cursor: usize, down: bool, len: usize) -> usize {
    let Some(max) = len.checked_sub(1) else {
        return 0;
    };
    if down {
        cursor.saturating_add(1).min(max)
    } else {
        cursor.saturating_sub(1)
    }
}

fn toggle_current(state: &mut AppState) {
    match state.focus {
        Focus::Categories => toggle_cat(state),
        Focus::Subsystems => {
            if let Some(sub) = state.subsystems.get_mut(state.sub_cursor) {
                sub.enabled = !sub.enabled;
            }
        }
        Focus::Exclude | Focus::Tree => {}
    }
}

fn toggle_cat(state: &mut AppState) {
    let cursor = state.cat_cursor;
    if cursor == 0 {
        state.all_collapsed = !state.all_collapsed;
        if state.all_collapsed {
            scanner::collapse_all(&mut state.tree);
        } else {
            scanner::expand_all(&mut state.tree);
        }
        return;
    }
    if cursor == 1 {
        let target = !state.are_all_selected();
        for cat in &mut state.categories {
            cat.enabled = target;
        }
        return;
    }
    let base = cursor - 2;
    if state.has_gitignore {
        if base == 0 {
            state.respect_gitignore = !state.respect_gitignore;
        } else if let Some(cat) = state.categories.get_mut(base - 1) {
            cat.enabled = !cat.enabled;
        }
    } else if let Some(cat) = state.categories.get_mut(base) {
        cat.enabled = !cat.enabled;
    }
}

fn toggle_gitignore(state: &mut AppState) {
    if state.has_gitignore {
        state.respect_gitignore = !state.respect_gitignore;
    }
}

fn exclude_at_cursor(state: &mut AppState) {
    let flat = scanner::flatten_visible(&state.tree);
    if let Some(item) = flat.get(state.tree_cursor()) {
        state.exclude.toggle(item.path.clone());
    }
}

fn unexclude_current(state: &mut AppState) {
    if state.focus != Focus::Exclude {
        return;
    }
    let path: Option<PathBuf> = state.exclude.items().get(state.exclude_cursor).cloned();
    if let Some(p) = path {
        state.exclude.toggle(p);
    }
}

fn toggle_expand(state: &mut AppState) {
    if state.focus != Focus::Tree {
        return;
    }
    let flat = scanner::flatten_visible(&state.tree);
    if let Some(item) = flat.get(state.tree_cursor()) {
        if item.is_dir {
            scanner::toggle_node_expanded(&mut state.tree, &item.path.clone());
        }
    }
}

fn do_clipboard(state: &mut AppState) -> Result<()> {
    let content = packer::build_export(state)?;
    let n = state.selected_paths().len();
    output::to_clipboard(&content)?;
    state.status = format!("✓ Copied {n} files");
    Ok(())
}

fn do_file(state: &mut AppState) -> Result<()> {
    let content = packer::build_export(state)?;
    let root_name = state
        .root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("export");
    let filename = format!("{root_name}_pakr.txt");
    let path = state.root.join(&filename);
    output::to_file(&content, &path)?;
    state.status = format!("✓ Saved {filename}");
    Ok(())
}
