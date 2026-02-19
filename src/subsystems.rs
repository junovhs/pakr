use crate::{semmap::SemmapData, types::Subsystem};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

pub fn build(data: &SemmapData) -> Vec<Subsystem> {
    find_components(data)
        .into_iter()
        .filter(|c| c.len() >= 2)
        .enumerate()
        .map(|(i, files)| Subsystem {
            name: cluster_name(&files, i),
            files,
            enabled: true,
        })
        .collect()
}

fn find_components(data: &SemmapData) -> Vec<Vec<PathBuf>> {
    let adj = build_adj(data);
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut components: Vec<Vec<PathBuf>> = Vec::new();
    for file in data.all_files() {
        if !visited.contains(&file) {
            components.push(bfs(&file, &adj, &mut visited));
        }
    }
    components
}

fn build_adj(data: &SemmapData) -> HashMap<PathBuf, Vec<PathBuf>> {
    let mut adj: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    for edge in &data.edges {
        adj.entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
        adj.entry(edge.to.clone())
            .or_default()
            .push(edge.from.clone());
    }
    adj
}

fn bfs(
    start: &Path,
    adj: &HashMap<PathBuf, Vec<PathBuf>>,
    visited: &mut HashSet<PathBuf>,
) -> Vec<PathBuf> {
    let mut queue = vec![start.to_path_buf()];
    let mut component = Vec::new();
    while let Some(node) = queue.pop() {
        if !visited.insert(node.clone()) {
            continue;
        }
        component.push(node.clone());
        if let Some(neighbors) = adj.get(&node) {
            for n in neighbors {
                if !visited.contains(n) {
                    queue.push(n.clone());
                }
            }
        }
    }
    component
}

fn cluster_name(files: &[PathBuf], index: usize) -> String {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for file in files {
        if let Some(dir) = file
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
        {
            *counts.entry(dir).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, v)| *v)
        .map_or_else(|| format!("cluster_{index}"), |(k, _)| k.to_string())
}
