use anyhow::Result;
use arboard::Clipboard;
use std::{fs, path::Path};

pub fn to_clipboard(content: &str) -> Result<()> {
    let mut cb = Clipboard::new()?;
    cb.set_text(content)?;
    Ok(())
}

pub fn to_file(content: &str, path: &Path) -> Result<()> {
    fs::write(path, content)?;
    Ok(())
}
