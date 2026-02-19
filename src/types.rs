use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

/// A node in the scanned directory tree.
#[derive(Debug, Clone)]
pub struct FileNode {
    /// Path relative to project root.
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    /// Bytes (0 for directories).
    pub size: u64,
    pub children: Vec<FileNode>,
    pub expanded: bool,
}

impl FileNode {
    pub fn total_size(&self) -> u64 {
        if self.is_dir {
            self.children.iter().map(FileNode::total_size).sum()
        } else {
            self.size
        }
    }

    pub fn file_count(&self) -> usize {
        if self.is_dir {
            self.children.iter().map(FileNode::file_count).sum()
        } else {
            1
        }
    }
}

/// Categorisation source for a file group.
#[derive(Debug, Clone)]
pub enum CategoryKind {
    SemmapLayer { index: u8, label: String },
    Source,
    Config,
    Docs,
    Build,
    Assets,
}

impl CategoryKind {
    pub fn display_name(&self) -> &str {
        match self {
            Self::SemmapLayer { label, .. } => label.as_str(),
            Self::Source => "Source",
            Self::Config => "Config",
            Self::Docs => "Docs",
            Self::Build => "Build & Generated",
            Self::Assets => "Assets & Binary",
        }
    }
}

/// A toggleable group of files.
#[derive(Debug, Clone)]
pub struct Category {
    pub kind: CategoryKind,
    pub files: Vec<PathBuf>,
    pub enabled: bool,
}

impl Category {
    pub fn name(&self) -> &str {
        self.kind.display_name()
    }

    pub fn token_estimate(&self, sizes: &HashMap<PathBuf, u64>) -> usize {
        self.files
            .iter()
            .filter_map(|p| sizes.get(p))
            .map(|&s| usize::try_from(s / 4).unwrap_or(usize::MAX / 4))
            .sum()
    }
}

/// A semantic cluster of related files.
#[derive(Debug, Clone)]
pub struct Subsystem {
    pub name: String,
    pub files: Vec<PathBuf>,
    pub enabled: bool,
}

impl Subsystem {
    pub fn token_estimate(&self, sizes: &HashMap<PathBuf, u64>) -> usize {
        self.files
            .iter()
            .filter_map(|p| sizes.get(p))
            .map(|&s| usize::try_from(s / 4).unwrap_or(usize::MAX / 4))
            .sum()
    }
}

/// Files explicitly excluded from export.
#[derive(Debug, Clone, Default)]
pub struct NixList(Vec<PathBuf>);

impl NixList {
    pub fn toggle(&mut self, path: PathBuf) {
        if let Some(pos) = self.0.iter().position(|p| p == &path) {
            self.0.remove(pos);
        } else {
            self.0.push(path);
        }
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.0.iter().any(|p| p.as_path() == path)
    }

    pub fn items(&self) -> &[PathBuf] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Which TUI panel section has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Categories,
    Subsystems,
    Nix,
    Tree,
}

/// Top-level application state.
#[derive(Debug)]
pub struct AppState {
    pub root: PathBuf,
    pub tree: FileNode,
    pub file_sizes: HashMap<PathBuf, u64>,
    pub categories: Vec<Category>,
    pub subsystems: Vec<Subsystem>,
    pub nix: NixList,
    pub has_semmap: bool,
    pub focus: Focus,
    pub cat_cursor: usize,
    pub sub_cursor: usize,
    pub nix_cursor: usize,
    pub tree_cursor: usize,
    pub tree_scroll: usize,
    pub status: String,
}

impl AppState {
    /// Compute the set of files to export based on current selections.
    pub fn selected_paths(&self) -> Vec<PathBuf> {
        let mut result: Vec<PathBuf> = self
            .categories
            .iter()
            .filter(|c| c.enabled)
            .flat_map(|c| c.files.iter().cloned())
            .collect();

        if !self.subsystems.is_empty() {
            let sub_set: HashSet<&PathBuf> = self
                .subsystems
                .iter()
                .filter(|s| s.enabled)
                .flat_map(|s| s.files.iter())
                .collect();
            result.retain(|p| sub_set.contains(p));
        }

        result.retain(|p| !self.nix.contains(p.as_path()));
        result.sort();
        result.dedup();
        result
    }

    pub fn total_tokens(&self) -> usize {
        self.selected_paths()
            .iter()
            .filter_map(|p| self.file_sizes.get(p))
            .map(|&s| usize::try_from(s / 4).unwrap_or(usize::MAX / 4))
            .sum()
    }

    pub fn total_bytes(&self) -> u64 {
        self.selected_paths()
            .iter()
            .filter_map(|p| self.file_sizes.get(p))
            .sum()
    }

    pub fn clamp_cursors(&mut self) {
        self.cat_cursor = self.cat_cursor.min(self.categories.len().saturating_sub(1));
        self.sub_cursor = self.sub_cursor.min(self.subsystems.len().saturating_sub(1));
        self.nix_cursor = self
            .nix_cursor
            .min(self.nix.items().len().saturating_sub(1));
    }
}
