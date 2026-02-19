use crate::types::FileNode;
use anyhow::Result;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

const IGNORE_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
    "__pycache__",
    "coverage",
    ".cache",
];

pub struct ScanResult {
    pub tree: FileNode,
    pub file_sizes: HashMap<PathBuf, u64>,
}

#[derive(Debug, Clone)]
pub struct FlatItem {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub expanded: bool,
    pub depth: usize,
}

pub fn scan(root: &Path) -> Result<ScanResult> {
    let mut file_sizes = HashMap::new();
    let tree = walk(root, root, &mut file_sizes)?;
    Ok(ScanResult { tree, file_sizes })
}

fn walk(root: &Path, path: &Path, sizes: &mut HashMap<PathBuf, u64>) -> Result<FileNode> {
    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or(".")
        .to_string();
    let meta = fs::metadata(path)?;

    if meta.is_file() {
        let size = meta.len();
        let rel = relative_path(root, path);
        sizes.insert(rel.clone(), size);
        return Ok(FileNode {
            path: rel,
            name,
            is_dir: false,
            size,
            children: Vec::new(),
            expanded: false,
        });
    }

    let mut children: Vec<FileNode> = fs::read_dir(path)?
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            let s = e.file_name().to_string_lossy().into_owned();
            !s.starts_with('.') || s == ".gitignore" || s == ".slopchopignore"
        })
        .filter(|e| {
            let s = e.file_name().to_string_lossy().into_owned();
            !IGNORE_DIRS.contains(&s.as_str())
        })
        .filter_map(|e| walk(root, &e.path(), sizes).ok())
        .collect();

    children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    let rel = relative_path(root, path);
    Ok(FileNode {
        path: rel,
        name,
        is_dir: true,
        size: 0,
        children,
        expanded: true,
    })
}

fn relative_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

pub fn all_files(node: &FileNode) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_files(node, &mut out);
    out
}

fn collect_files(node: &FileNode, out: &mut Vec<PathBuf>) {
    if node.is_dir {
        for child in &node.children {
            collect_files(child, out);
        }
    } else {
        out.push(node.path.clone());
    }
}

pub fn flatten_visible(node: &FileNode) -> Vec<FlatItem> {
    let mut out = Vec::new();
    push_flat(node, 0, &mut out);
    out
}

fn push_flat(node: &FileNode, depth: usize, out: &mut Vec<FlatItem>) {
    out.push(FlatItem {
        path: node.path.clone(),
        name: node.name.clone(),
        is_dir: node.is_dir,
        expanded: node.expanded,
        depth,
    });
    if node.is_dir && node.expanded {
        for child in &node.children {
            push_flat(child, depth + 1, out);
        }
    }
}

pub fn toggle_node_expanded(tree: &mut FileNode, path: &Path) {
    if tree.path.as_path() == path {
        tree.expanded = !tree.expanded;
        return;
    }
    for child in &mut tree.children {
        toggle_node_expanded(child, path);
    }
}
