use anyhow::Result;
use clap::Parser;
use ignore::gitignore::GitignoreBuilder;
use pakr::{categories, scanner, semmap, subsystems, types};
use ratatui::{layout::Rect, widgets::ListState};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "pakr", about = "Terminal file packer for AI context")]
struct Args {
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let root = args.path.canonicalize()?;

    let scan = scanner::scan(&root)?;
    let semmap_data = semmap::load(&root).ok();
    let has_semmap = semmap_data.is_some();

    let (cats, subs) = if let Some(ref sd) = semmap_data {
        (
            categories::from_semmap(sd, &scan.tree),
            subsystems::build(sd),
        )
    } else {
        (categories::from_heuristics(&scan.tree), Vec::new())
    };

    let gi_path = root.join(".gitignore");
    let has_gitignore = gi_path.exists();
    let gitignore_filter = if has_gitignore {
        let mut builder = GitignoreBuilder::new(&root);
        builder.add(&gi_path);
        builder
            .build()
            .ok()
            .map(types::GitignoreFilter::new)
            .unwrap_or_default()
    } else {
        types::GitignoreFilter::default()
    };

    let state = types::AppState {
        root,
        tree: scan.tree,
        file_sizes: scan.file_sizes,
        categories: cats,
        subsystems: subs,
        exclude: types::ExcludeList::default(),
        manual_includes: Vec::new(),
        has_semmap,
        has_gitignore,
        respect_gitignore: has_gitignore,
        gitignore_filter,
        all_collapsed: false,
        focus: types::Focus::Categories,
        cat_cursor: 0,
        sub_cursor: 0,
        exclude_cursor: 0,
        hover_path: None,
        input_mode: false,
        input_buffer: String::new(),
        cat_list_state: ListState::default(),
        sub_list_state: ListState::default(),
        tree_list_state: ListState::default(),
        cat_area: Rect::default(),
        sub_area: Rect::default(),
        tree_area: Rect::default(),
        status: String::from("ready"),
    };

    pakr::tui::run(state)
}
