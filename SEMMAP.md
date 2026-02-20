# project -- Semantic Map

## Legend

`[ENTRY]` Application entry point

`[CORE]` Core business logic

`[TYPE]` Data structures and types

`[UTIL]` Utility functions

## Layer 0 -- Config

`Cargo.toml`
Rust package manifest and dependencies. Centralizes project configuration.

## Layer 1 -- Core

`src/lib.rs`
Library root and public exports. Provides application entry point.

`src/main.rs`
Orchestrates `anyhow`, `clap`, `ignore`. Provides application entry point.

`src/tui/mod.rs`
Module providing `run`. Supports application functionality.
→ Exports: run

`src/tui/panels/mod.rs`
Module definitions for mod. Supports application functionality.

## Layer 2 -- Domain

`src/categories.rs`
Module providing `from_heuristics`, `from_semmap`. Supports application functionality.
→ Exports: from_heuristics, from_semmap

`src/output.rs`
Module providing `to_clipboard`, `to_file`. Supports application functionality.
→ Exports: to_clipboard, to_file

`src/packer.rs`
Module providing `build_export`. Supports application functionality.
→ Exports: build_export

`src/scanner.rs`
Module providing `FlatItem`, `ScanResult`, `all_files`. Supports application functionality.
→ Exports: FlatItem, ScanResult, all_files, collapse_all, expand_all, flatten_visible, scan, toggle_node_expanded

`src/semmap.rs`
Build a map from Mermaid node id (e.g. Supports application functionality.
→ Exports: DepEdge, Layer, SemmapData, adjacency, all_files, load

`src/subsystems.rs`
Module providing `build`. Supports application functionality.
→ Exports: build

`src/tui/keys.rs`
Module providing `handle_key`, `handle_mouse`. Supports application functionality.
→ Exports: handle_key, handle_mouse

`src/tui/layout.rs`
Module providing `render`. Supports application functionality.
→ Exports: render

`src/tui/panels/left.rs`
Module providing `render`. Supports application functionality.
→ Exports: render

`src/tui/panels/right.rs`
Module providing `render`. Supports application functionality.
→ Exports: render

`src/types.rs`
2 control rows + optional gitignore row + category rows. Defines domain data structures.
→ Exports: AppState, Category, CategoryKind, ExcludeList, FileNode, Focus, GitignoreFilter, Subsystem, are_all_selected, cat_list_len, clamp_cursors, contains, display_name, has_filter, is_empty, is_ignored, items, name, new, selected_paths, set_tree_cursor, toggle, token_estimate, total_bytes, total_tokens, tree_cursor

