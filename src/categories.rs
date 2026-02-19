use crate::{
    scanner,
    semmap::SemmapData,
    types::{Category, CategoryKind},
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

pub fn from_semmap(data: &SemmapData, tree: &crate::types::FileNode) -> Vec<Category> {
    let mut cats: Vec<Category> = data
        .layers
        .iter()
        .map(|layer| Category {
            kind: CategoryKind::SemmapLayer {
                index: layer.index,
                label: layer.label.clone(),
            },
            files: layer.files.clone(),
            enabled: is_layer_on_by_default(layer.index),
        })
        .collect();

    // Files on disk not mentioned in any SEMMAP layer get an "Other" category.
    let tracked: HashSet<&PathBuf> = data.layers.iter().flat_map(|l| l.files.iter()).collect();

    let untracked: Vec<PathBuf> = scanner::all_files(tree)
        .into_iter()
        .filter(|p| !tracked.contains(p))
        .collect();

    if !untracked.is_empty() {
        // Split untracked into docs vs other
        let docs: Vec<PathBuf> = untracked.iter().filter(|p| is_doc(p)).cloned().collect();
        let other: Vec<PathBuf> = untracked.into_iter().filter(|p| !is_doc(p)).collect();

        if !docs.is_empty() {
            cats.push(Category {
                kind: CategoryKind::Docs,
                files: docs,
                enabled: false,
            });
        }
        if !other.is_empty() {
            cats.push(Category {
                kind: CategoryKind::Assets,
                files: other,
                enabled: false,
            });
        }
    }

    cats
}

fn is_layer_on_by_default(index: u8) -> bool {
    index <= 2
}

pub fn from_heuristics(tree: &crate::types::FileNode) -> Vec<Category> {
    let all = scanner::all_files(tree);

    let source: Vec<PathBuf> = all.iter().filter(|p| is_source(p)).cloned().collect();
    let config: Vec<PathBuf> = all.iter().filter(|p| is_config(p)).cloned().collect();
    let docs: Vec<PathBuf> = all.iter().filter(|p| is_doc(p)).cloned().collect();
    let build: Vec<PathBuf> = all.iter().filter(|p| is_build(p)).cloned().collect();
    let assets: Vec<PathBuf> = all
        .iter()
        .filter(|p| !is_source(p) && !is_config(p) && !is_doc(p) && !is_build(p))
        .cloned()
        .collect();

    vec![
        Category {
            kind: CategoryKind::Source,
            files: source,
            enabled: true,
        },
        Category {
            kind: CategoryKind::Config,
            files: config,
            enabled: true,
        },
        Category {
            kind: CategoryKind::Docs,
            files: docs,
            enabled: false,
        },
        Category {
            kind: CategoryKind::Build,
            files: build,
            enabled: false,
        },
        Category {
            kind: CategoryKind::Assets,
            files: assets,
            enabled: false,
        },
    ]
}

fn ext(p: &Path) -> &str {
    p.extension().and_then(|e| e.to_str()).unwrap_or("")
}

fn is_source(p: &Path) -> bool {
    matches!(
        ext(p),
        "rs" | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "py"
            | "go"
            | "c"
            | "cpp"
            | "h"
            | "cs"
            | "swift"
            | "kt"
            | "ex"
            | "exs"
            | "hs"
            | "nim"
            | "zig"
    )
}

fn is_config(p: &Path) -> bool {
    let name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(ext(p), "toml" | "json" | "yaml" | "yml" | "ini" | "cfg")
        || matches!(
            name.as_str(),
            "dockerfile" | ".eslintrc" | ".prettierrc" | ".babelrc"
        )
}

fn is_doc(p: &Path) -> bool {
    let name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(ext(p), "md" | "rst" | "txt" | "adoc")
        || name.starts_with("readme")
        || name.starts_with("changelog")
        || name.starts_with("license")
}

fn is_build(p: &Path) -> bool {
    let name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(ext(p), "lock") || name.ends_with("-lock.json") || name == "package-lock.json"
}
