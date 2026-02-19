use anyhow::{bail, Result};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Layer {
    pub index: u8,
    pub label: String,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct DepEdge {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SemmapData {
    pub project_name: String,
    pub layers: Vec<Layer>,
    pub edges: Vec<DepEdge>,
}

impl SemmapData {
    pub fn all_files(&self) -> Vec<PathBuf> {
        self.layers
            .iter()
            .flat_map(|l| l.files.iter().cloned())
            .collect()
    }

    pub fn adjacency(&self) -> HashMap<PathBuf, Vec<PathBuf>> {
        let mut map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
        for edge in &self.edges {
            map.entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
        }
        map
    }
}

pub fn load(root: &Path) -> Result<SemmapData> {
    let path = root.join("SEMMAP.md");
    if !path.exists() {
        bail!("SEMMAP.md not found");
    }
    let text = std::fs::read_to_string(&path)?;
    Ok(parse(&text))
}

fn parse(text: &str) -> SemmapData {
    let (project_name, layers) = parse_layers(text);
    let edges = parse_edges(text, &layers);
    SemmapData {
        project_name,
        layers,
        edges,
    }
}

fn parse_layers(text: &str) -> (String, Vec<Layer>) {
    let mut project_name = String::new();
    let mut layers: Vec<Layer> = Vec::new();
    let mut current: Option<Layer> = None;

    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("# project --") {
            project_name = rest.trim().to_string();
        } else if let Some(rest) = t.strip_prefix("## Layer ") {
            if let Some(prev) = current.take() {
                layers.push(prev);
            }
            current = parse_layer_header(rest);
        } else if let Some(layer) = current.as_mut() {
            try_push_path(t, layer);
        }
    }
    if let Some(prev) = current.take() {
        layers.push(prev);
    }
    (project_name, layers)
}

fn parse_layer_header(rest: &str) -> Option<Layer> {
    let (idx_str, label) = rest.split_once(" -- ")?;
    let index: u8 = idx_str.trim().parse().ok()?;
    Some(Layer {
        index,
        label: label.trim().to_string(),
        files: Vec::new(),
    })
}

fn try_push_path(t: &str, layer: &mut Layer) {
    if t.starts_with('`') && t.ends_with('`') && t.len() > 2 {
        let inner = &t[1..t.len() - 1];
        if looks_like_path(inner) {
            layer.files.push(PathBuf::from(inner));
        }
    }
}

fn parse_edges(text: &str, layers: &[Layer]) -> Vec<DepEdge> {
    if !text.contains("graph TD") {
        return Vec::new();
    }
    let all_files: Vec<PathBuf> = layers
        .iter()
        .flat_map(|l| l.files.iter().cloned())
        .collect();
    let node_map = build_node_map(&all_files);

    text.lines()
        .filter_map(|line| parse_edge_line(line.trim(), &node_map))
        .collect()
}

fn parse_edge_line(trimmed: &str, node_map: &HashMap<String, PathBuf>) -> Option<DepEdge> {
    let (from_id, to_id) = trimmed.split_once(" --> ")?;
    let from = node_map.get(from_id.trim())?.clone();
    let to = node_map.get(to_id.trim())?.clone();
    Some(DepEdge { from, to })
}

/// Build a map from Mermaid node id (e.g. `src_foo_rs`) → `PathBuf`.
fn build_node_map(files: &[PathBuf]) -> HashMap<String, PathBuf> {
    files
        .iter()
        .map(|p| {
            let id = p.to_string_lossy().replace(['/', '\\', '.', '-'], "_");
            (id, p.clone())
        })
        .collect()
}

fn looks_like_path(s: &str) -> bool {
    s.contains('.') || s.contains('/') || s.contains('\\')
}
