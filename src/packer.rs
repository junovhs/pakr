use crate::types::AppState;
use anyhow::Result;
use std::{fmt::Write as _, fs, path::Path};

pub fn build_export(state: &AppState) -> Result<String> {
    let name = project_name(&state.root);
    let mut out = format!("// PAKR COMBINED TEXT EXPORT //\n// Project: {name}\n\n");
    for path in state.selected_paths() {
        let full = state.root.join(&path);
        let display = path.display().to_string();
        match fs::read_to_string(&full) {
            Ok(content) => append_file(&mut out, &display, &content),
            Err(e) => {
                let _ = writeln!(out, "// ERROR: {display}: {e}");
                out.push('\n');
            }
        }
    }
    Ok(out)
}

fn project_name(root: &Path) -> String {
    root.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string()
}

fn append_file(out: &mut String, path: &str, content: &str) {
    let _ = writeln!(out, "// ===== START: {path} =====");
    out.push_str(content);
    if !content.ends_with('\n') {
        out.push('\n');
    }
    let _ = writeln!(out, "// ===== END: {path} =====");
    out.push('\n');
}
