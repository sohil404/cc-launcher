# CC Launcher

One-click launcher for [Claude Code](https://docs.anthropic.com/en/docs/claude-code) in your project directories.

Lightweight menu bar app (~8MB) built with Tauri v2. No Electron, no bloat.

## Features

- **Auto-discovers projects** — scans directories for `.git`, `CLAUDE.md`, `package.json`, `pyproject.toml`, `Cargo.toml`
- **One-click launch** — opens your terminal with `claude` and your configured flags
- **Resume sessions** — resume any previous Claude session with `--resume` (Claude's session picker)
- **Menu bar app** — lives in your system tray, launch projects without opening a window
- **Claude metadata** — shows session count, message count, and last active time per project (read from `~/.claude/`)
- **Pin favorites** — star projects to keep them at the top
- **Search** — filter projects instantly with `⌘K` / `Ctrl+K`
- **Multi-select flags** — configure launch flags visually (model, permission mode, effort, etc.)
- **Smart filtering** — skips `node_modules`, `worktrees`, and common subfolders like `frontend/` within projects
- **Configurable** — scan directories, terminal, launch flags, scan depth

## Prerequisites

- [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) must be installed and on your PATH

## Install

Download the latest release for your platform from [Releases](https://github.com/sohil404/cc-launcher/releases).

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | `.dmg` |
| macOS (Intel) | `.dmg` |
| Linux | `.AppImage` / `.deb` |
| Windows | `.msi` / `.exe` |

## Getting Started

1. **Open the app** — it starts in your menu bar (look for the CC icon in the system tray)
2. **Click the tray icon** — you'll see your pinned and recent projects listed
3. **Click "Show Window"** — opens the full UI with search, settings, and all projects
4. **Add scan directories** — go to Settings and add paths where your projects live (e.g., `~/Projects`)
5. **Launch a project** — click any project to open Claude Code in your terminal
6. **Pin favorites** — click the star icon on projects you use most

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `⌘K` / `Ctrl+K` | Focus search |
| `↑` `↓` | Navigate projects |
| `Enter` | Launch selected project |
| `Escape` | Clear search |
| `⌘,` / `Ctrl+,` | Open settings |

## Terminal Support

| OS | Supported Terminals |
|----|---------------------|
| macOS | iTerm2, Warp, Alacritty, Kitty, Terminal.app |
| Linux | GNOME Terminal, Konsole, Alacritty, Kitty, Xfce Terminal, WezTerm |
| Windows | Windows Terminal, PowerShell, cmd.exe |

Auto-detect picks the best available, or choose manually in settings.

## Build from Source

### Prerequisites

- [Rust](https://rustup.rs/) 1.77+
- Tauri CLI: `cargo install tauri-cli --version "^2" --locked`

### Build

```bash
git clone https://github.com/sohil404/cc-launcher.git
cd cc-launcher
cargo tauri build
```

The built app will be in `src-tauri/target/release/bundle/`.

### Development

```bash
cargo tauri dev
```

## Configuration

Config is stored at `~/.cc-launcher/config.json`:

```json
{
  "scan_dirs": ["~/Desktop", "~/Projects"],
  "pinned": [],
  "recents": [],
  "launch_flags": "",
  "max_recents": 20,
  "scan_depth": 3,
  "terminal": "auto"
}
```

| Field | Description |
|-------|-------------|
| `scan_dirs` | Directories to scan for projects |
| `pinned` | Paths of pinned projects |
| `recents` | Recently launched projects with timestamps |
| `launch_flags` | Flags passed to `claude` (e.g., `--model opus --effort high`) |
| `max_recents` | Maximum number of recent entries to keep |
| `scan_depth` | How deep to scan inside each directory |
| `terminal` | Terminal to use: `auto`, `iterm2`, `warp`, `alacritty`, `kitty`, `terminal` |

## How It Works

1. Scans configured directories for project markers (`.git`, `CLAUDE.md`, `package.json`, etc.)
2. Reads `~/.claude/history.jsonl` and `~/.claude/projects/` for per-project Claude metadata
3. Lists projects sorted by: pinned → recently active → alphabetical
4. Click "open" → opens your terminal running `cd <project> && claude <flags>`
5. Click "resume" → opens terminal with `claude --resume` to pick from previous sessions

## Contributing

PRs welcome. The codebase is small:

```
src/                    # Frontend (vanilla HTML/CSS/JS)
src-tauri/src/
  ├── lib.rs            # Tauri setup, tray menu
  ├── commands.rs       # Tauri commands (frontend ↔ backend)
  ├── scanner.rs        # Project discovery
  ├── config.rs         # Config persistence
  ├── launcher.rs       # Terminal spawning (cross-platform)
  └── claude_meta.rs    # Claude session metadata reader
```

## License

MIT
