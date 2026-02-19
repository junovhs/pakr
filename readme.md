# pakr

**Terminal file packer for AI context. Select exactly what to share, export in one keystroke.**

You're in your terminal working on a project. You need to give an AI the right files — not everything, not a manual list, just the *relevant* stuff. pakr gets you there in seconds with a live TUI, semantic-aware categorization, and direct clipboard export.

No browser. No drag and drop. No `cat`-ing files together manually.

---

## Install

```bash
git clone https://github.com/yourname/pakr
cd pakr
cargo install --path . --force
```

---

## Usage

```bash
pakr                  # analyze current directory
pakr ./my-project     # analyze a specific directory
```

That's it. pakr opens a TUI, scans your project, and you start selecting.

---

## The Interface

```
┌─ INCLUDE ────────────────┐ ┌─ PREVIEW TREE ───────────────────────────────┐
│   ▼ COLLAPSE ALL         │ │   ▼ my-project/                              │
│   ☑ DESELECT ALL         │ │     ▼ src/                                   │
│ [✓] 🚫 .gitignore        │ │       ✓   main.rs              ~1k           │
│ [✓] Layer 0 — Config ~1k │ │       ✓   lib.rs               ~2k           │
│ [✓] Layer 1 — Core  ~14k │ │       ✓   types.rs             ~3k           │
│ [✓] Layer 2 — Domain ~46k│ │     ▼ tests/                                 │
│ [✗] Tests          ~5k   │ │         integration.rs         ~1k           │
│ [✗] Docs           ~12k  │ └──────────────────────────────────────────────┘
│ [✗] Assets & Binary ~46k │
├─ SUMMARY ────────────────┤
│ 18 files  84 KB  ~28k tok│
│                          │
│ EXCLUDED  [e]focus [d]rm │
│  nothing excluded        │
│                          │
│ [↑↓]nav [spc]toggle [tab]│
│ [n]excl [a]add [↵]copy   │
└──────────────────────────┘
```

**Left panel** — your selection controls. Always visible, never truncated.  
**Right panel** — live tree showing exactly what's included (✓) or excluded (✗).

---

## SEMMAP Integration

If your project has a `SEMMAP.md` file, pakr reads it automatically and replaces the heuristic categories with your actual semantic layers:

```
[✓] Layer 0 — Config    ~1k tok
[✓] Layer 1 — Core     ~14k tok
[✓] Layer 2 — Domain   ~46k tok
[✗] Layer 3 — Utilities  ~3k tok
[✗] Layer 4 — Tests      ~5k tok
[✓] Docs                 ~8k tok    ← untracked files get their own category
[✗] Assets & Binary     ~46k tok
```

Layers 0–2 are enabled by default. Tests and utilities off by default. Files not mentioned in any SEMMAP layer are caught in automatic "Docs" or "Assets" overflow categories.

If there's no `SEMMAP.md`, pakr falls back to heuristic detection (Source / Config / Docs / Build / Assets).

---

## Keyboard Reference

### Navigation

| Key | Action |
|-----|--------|
| `↑` `↓` | Move cursor |
| `Tab` | Cycle focus between panels |
| `Esc` | Return to categories panel |

### Selection

| Key | Action |
|-----|--------|
| `Space` | Toggle selected item on/off |
| `Space` on COLLAPSE ALL | Toggle all folders collapsed/expanded |
| `Space` on SELECT ALL | Toggle all categories on/off |
| `Space` on .gitignore row | Toggle gitignore filtering on/off |
| `g` | Toggle gitignore filtering (shortcut, any panel) |

### Tree panel

| Key | Action |
|-----|--------|
| `←` `→` | Collapse/expand folder at cursor |
| `n` | Exclude file/folder at tree cursor |
| `Tab` to tree, then `↑↓` | Navigate tree items |

### Exclude panel

| Key | Action |
|-----|--------|
| `e` | Focus the exclude list |
| `d` or `Delete` | Remove item from exclude list |

### Manual adds

| Key | Action |
|-----|--------|
| `a` | Open input bar to add a file by path |
| Type relative path, `Enter` | Confirm add |
| `Esc` | Cancel |

### Export

| Key | Action |
|-----|--------|
| `Enter` | Copy combined export to clipboard |
| `f` | Save combined export to `{project}_pakr.txt` in project root |
| `q` | Quit |

### Mouse

| Action | Effect |
|--------|--------|
| Click item in tree | Focus tree + move cursor to that item |
| Click folder toggle (▼/▶) | Collapse/expand that folder |
| Click item in INCLUDE | Focus panel + move cursor |
| Hover over tree | Highlights item in cyan |

---

## How Selection Works

pakr uses a layered selection model:

1. **Categories** determine the base file set (SEMMAP layers or heuristics)
2. **Subsystems** (if SEMMAP present) act as a filter — only files in enabled subsystems are kept
3. **Exclude list** removes specific files regardless of categories
4. **Manual adds** bypass all filters — always included unless explicitly excluded
5. **Gitignore** filters out ignored files when enabled

The live tree and token count update instantly as you change any of these.

---

## Token Counts

Tokens are estimated at `bytes / 3` — this errs slightly high, which is what you want when planning context. The count shown in the SUMMARY panel and next to each file in the tree reflects only the currently selected set.

```
18 files  84 KB  ~28k tok
```

File-level counts appear inline in the tree:

```
✓   main.rs    ~1k
✓   types.rs   ~4k
```

---

## Export Format

pakr outputs a single concatenated text file with clear file delimiters:

```
// PAKR COMBINED TEXT EXPORT //
// Project: my-project

// ===== START: src/main.rs =====
fn main() {
    ...
}
// ===== END: src/main.rs =====

// ===== START: src/types.rs =====
...
```

Paste directly into Claude, ChatGPT, or any AI context window. The delimiters make it easy for the AI to understand where each file begins and ends.

---

## .gitignore Support

When pakr detects a `.gitignore` in the project root, it adds a toggleable row at the top of the INCLUDE panel. When enabled (default), gitignored files are filtered out of the selection even if their category is on. Toggle with `Space` on that row or press `g` from anywhere.

---

## The Exclude List

Excluding a file or folder removes it from the export regardless of what categories are enabled. Useful for:

- A huge generated file in an otherwise-useful directory
- A secrets file that snuck into source
- A folder you want to keep collapsed but not export

Navigate to the tree, move cursor to the item, press `n`. To remove from the exclude list: press `e` to focus it, navigate to the item, press `d`.

---

## Manual Adds

Sometimes you need a file that pakr's categories don't include — a config file outside the normal structure, a one-off doc, anything. Press `a`, type the relative path from the project root, press `Enter`. It shows up highlighted in green in the SUMMARY panel and is always included in the export.

---

## Tips

**For sharing a codebase with an AI:** Enable your core source layers, disable tests and docs. Hit `Enter`. Done.

**For sharing a specific subsystem:** If you have SEMMAP, scroll to SUBSYSTEMS and disable everything except the cluster you're working on.

**For sharing just config files:** Disable Source, enable Config only. Token count drops immediately — confirm it's small enough, then export.

**For trimming a big export:** Enable everything you want, then tab to the tree and `n` the specific files you don't need. They'll show as ✗ and get skipped.

**Check the token count before exporting.** Claude's context window is large but not infinite. If you're seeing `~150k tok` you probably have Docs or Assets enabled by accident.

---

## Comparison

| Approach | Steps | Pain |
|----------|-------|------|
| Manual copy-paste | Open each file, copy, paste, repeat | Many steps, easy to miss files |
| `cat src/**/*.rs` | One command, wrong files | No selection control |
| DirAnalyze (web) | Open browser, navigate to site, drag folder, wait, click | Leaves terminal |
| **pakr** | `pakr .` → `Enter` | Done |

---

## Built With

- [ratatui](https://github.com/ratatui-org/ratatui) — TUI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) — terminal backend  
- [arboard](https://github.com/1Password/arboard) — clipboard
- [ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore) — gitignore parsing
- [clap](https://github.com/clap-rs/clap) — CLI args
- [SEMMAP](https://github.com/yourname/semmap) — semantic layer detection (optional)

---

## License

MIT
