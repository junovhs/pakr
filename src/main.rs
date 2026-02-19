use anyhow::Result;
use clap::Parser;
use pakr::{categories, scanner, semmap, subsystems, types};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "pakr", about = "Terminal file packer for AI context")]
struct Args {
    /// Directory to analyze (defaults to current directory)
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
        (categories::from_semmap(sd), subsystems::build(sd))
    } else {
        (categories::from_heuristics(&scan.tree), Vec::new())
    };

    let state = types::AppState {
        root,
        tree: scan.tree,
        file_sizes: scan.file_sizes,
        categories: cats,
        subsystems: subs,
        nix: types::NixList::default(),
        has_semmap,
        focus: types::Focus::Categories,
        cat_cursor: 0,
        sub_cursor: 0,
        nix_cursor: 0,
        tree_cursor: 0,
        tree_scroll: 0,
        status: String::from(
            "[space] toggle  [tab] switch panel  [n] nix  [enter] clipboard  [f] save  [q] quit",
        ),
    };

    pakr::tui::run(state)
}
