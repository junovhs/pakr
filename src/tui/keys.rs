use crate::{
    output, packer, scanner,
    types::{AppState, Focus},
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use std::path::PathBuf;

pub fn handle(key: KeyEvent, state: &mut AppState) -> Result<bool> {
    match key.code {
        KeyCode::Char('q' | 'Q') => return Ok(true),
        KeyCode::Tab => cycle_focus(state),
        KeyCode::Up => move_cursor(state, false),
        KeyCode::Down => move_cursor(state, true),
        KeyCode::Char(' ') => toggle_current(state),
        KeyCode::Char('n') => nix_at_cursor(state),
        KeyCode::Char('d') | KeyCode::Delete => unnix_current(state),
        KeyCode::Enter => do_clipboard(state)?,
        KeyCode::Char('f') => do_file(state)?,
        KeyCode::Left | KeyCode::Right => toggle_expand(state),
        _ => {}
    }
    state.clamp_cursors();
    Ok(false)
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
        Focus::Subsystems => Focus::Nix,
        Focus::Nix => Focus::Tree,
        Focus::Tree => Focus::Categories,
    };
}

fn move_cursor(state: &mut AppState, down: bool) {
    match state.focus {
        Focus::Categories => {
            state.cat_cursor = step(state.cat_cursor, down, state.categories.len());
        }
        Focus::Subsystems => {
            state.sub_cursor = step(state.sub_cursor, down, state.subsystems.len());
        }
        Focus::Nix => {
            state.nix_cursor = step(state.nix_cursor, down, state.nix.items().len());
        }
        Focus::Tree => {
            let len = scanner::flatten_visible(&state.tree).len();
            state.tree_cursor = step(state.tree_cursor, down, len);
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
        Focus::Categories => {
            if let Some(cat) = state.categories.get_mut(state.cat_cursor) {
                cat.enabled = !cat.enabled;
            }
        }
        Focus::Subsystems => {
            if let Some(sub) = state.subsystems.get_mut(state.sub_cursor) {
                sub.enabled = !sub.enabled;
            }
        }
        Focus::Nix | Focus::Tree => {}
    }
}

fn nix_at_cursor(state: &mut AppState) {
    let flat = scanner::flatten_visible(&state.tree);
    if let Some(item) = flat.get(state.tree_cursor) {
        state.nix.toggle(item.path.clone());
    }
}

fn unnix_current(state: &mut AppState) {
    if state.focus != Focus::Nix {
        return;
    }
    let path: Option<PathBuf> = state.nix.items().get(state.nix_cursor).cloned();
    if let Some(p) = path {
        state.nix.toggle(p);
    }
}

fn toggle_expand(state: &mut AppState) {
    if state.focus != Focus::Tree {
        return;
    }
    let flat = scanner::flatten_visible(&state.tree);
    if let Some(item) = flat.get(state.tree_cursor) {
        if item.is_dir {
            scanner::toggle_node_expanded(&mut state.tree, &item.path);
        }
    }
}

fn do_clipboard(state: &mut AppState) -> Result<()> {
    let content = packer::build_export(state)?;
    let n = state.selected_paths().len();
    output::to_clipboard(&content)?;
    state.status = format!("Copied {n} files to clipboard");
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
    state.status = format!("Saved to {filename}");
    Ok(())
}
