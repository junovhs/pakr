use ratatui::{layout::Rect, widgets::ListState};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct GitignoreFilter(Option<ignore::gitignore::Gitignore>);

impl std::fmt::Debug for GitignoreFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GitignoreFilter({})", self.0.is_some())
    }
}

impl GitignoreFilter {
    pub fn new(gi: ignore::gitignore::Gitignore) -> Self {
        Self(Some(gi))
    }
    pub fn has_filter(&self) -> bool {
        self.0.is_some()
    }
    pub fn is_ignored(&self, root: &Path, rel: &Path) -> bool {
        let Some(gi) = &self.0 else {
            return false;
        };
        gi.matched(root.join(rel), false).is_ignore()
    }
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub children: Vec<FileNode>,
    pub expanded: bool,
}

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
            .map(|&s| usize::try_from(s / 3).unwrap_or(usize::MAX / 3))
            .sum()
    }
}

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
            .map(|&s| usize::try_from(s / 3).unwrap_or(usize::MAX / 3))
            .sum()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExcludeList(Vec<PathBuf>);

impl ExcludeList {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Categories,
    Subsystems,
    Exclude,
    Tree,
}

#[derive(Debug)]
pub struct AppState {
    pub root: PathBuf,
    pub tree: FileNode,
    pub file_sizes: HashMap<PathBuf, u64>,
    pub categories: Vec<Category>,
    pub subsystems: Vec<Subsystem>,
    pub exclude: ExcludeList,
    pub manual_includes: Vec<PathBuf>,
    pub has_semmap: bool,
    pub has_gitignore: bool,
    pub respect_gitignore: bool,
    pub gitignore_filter: GitignoreFilter,
    pub all_collapsed: bool,
    pub focus: Focus,
    pub cat_cursor: usize,
    pub sub_cursor: usize,
    pub exclude_cursor: usize,
    pub hover_path: Option<PathBuf>,
    pub input_mode: bool,
    pub input_buffer: String,
    pub cat_list_state: ListState,
    pub sub_list_state: ListState,
    pub tree_list_state: ListState,
    pub cat_area: Rect,
    pub sub_area: Rect,
    pub tree_area: Rect,
    pub status: String,
}

impl AppState {
    pub fn tree_cursor(&self) -> usize {
        self.tree_list_state.selected().unwrap_or(0)
    }

    pub fn set_tree_cursor(&mut self, idx: usize) {
        self.tree_list_state.select(Some(idx));
    }

    /// 2 control rows + optional gitignore row + category rows.
    pub fn cat_list_len(&self) -> usize {
        2 + self.categories.len() + usize::from(self.has_gitignore)
    }

    pub fn are_all_selected(&self) -> bool {
        self.categories.iter().all(|c| c.enabled)
    }

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

        result.retain(|p| !self.exclude.contains(p.as_path()));

        if self.respect_gitignore && self.gitignore_filter.has_filter() {
            result.retain(|p| !self.gitignore_filter.is_ignored(&self.root, p));
        }

        for path in &self.manual_includes {
            if !self.exclude.contains(path.as_path()) {
                result.push(path.clone());
            }
        }

        result.sort();
        result.dedup();
        result
    }

    pub fn total_tokens(&self) -> usize {
        self.selected_paths()
            .iter()
            .filter_map(|p| self.file_sizes.get(p))
            .map(|&s| usize::try_from(s / 3).unwrap_or(usize::MAX / 3))
            .sum()
    }

    pub fn total_bytes(&self) -> u64 {
        self.selected_paths()
            .iter()
            .filter_map(|p| self.file_sizes.get(p))
            .sum()
    }

    pub fn clamp_cursors(&mut self) {
        self.cat_cursor = self.cat_cursor.min(self.cat_list_len().saturating_sub(1));
        self.sub_cursor = self.sub_cursor.min(self.subsystems.len().saturating_sub(1));
        self.exclude_cursor = self
            .exclude_cursor
            .min(self.exclude.items().len().saturating_sub(1));
        let cur = self.tree_list_state.selected().unwrap_or(0);
        self.tree_list_state.select(Some(cur));
    }
}
