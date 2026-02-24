# tuihub

A Rust + `ratatui` app-store style TUI for terminal apps.

## Features

- Top tabs: `All`, `Installed`, `Categories`
- Category sub-tabs shown only when `Categories` is active
- Search bar via `/`
- Multi-select via `Space`
- Install via `I` (OS/WSL-aware command from JSON)
- Uninstall via `U`
- Launch via `L` in a fresh detached `tmux` session
- Details panel + status bar + key hints
- Graceful external command execution (raw mode disabled, alt-screen exited) so `sudo` password prompts work normally

## Catalog file

Edit `data/apps.json` to add more TUI apps.

## Run

```bash
cargo run
```

## Keys

- `q`: quit
- `Tab` / `Shift+Tab`: cycle top tabs
- `Left` / `Right`: cycle category sub-tabs (when `Categories` tab is active)
- `Up/Down` or `j/k`: move list cursor
- `Space`: select/deselect app
- `/`: open search input
- `I`: install selected apps (or current row if none selected)
- `U`: uninstall selected apps (or current row if none selected)
- `L`: launch selected apps in tmux (or current row if none selected)
