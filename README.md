# CNC View TUI

Terminal-based CNC toolpath viewer for GRBL-style G-code. Parses G0/G1/G2/G3 moves, projects to a 3D view, and renders with a side-by-side code panel.

## Features

- 3D toolpath projection with side-view default
- Configurable keybindings and Catppuccin Mocha theme
- Animation (play/pause) for toolpath reveal
- Code panel with visual range selection to preview combined toolpath
- Built-in ignore rules for non-G-code words

## Build

```
cargo build -p cnc-view-tui
```

## Run

```
cargo run -p cnc-view-tui -- <path-to-gcode>
```

## Keybindings (default)

- View: `h/j/k/l` pan, `w/s/a/d` rotate, `+/-` zoom
- Reset: `r` (pan+zoom), `g` fit, `p` projection
- Animation: `space` play/pause
- File panel: `tab` focus toggle, `v` visual select, `↑/↓` line select, `PgUp/PgDn` scroll
- Help: `?`
- Quit: `q`

## Config

Config lookup:
- `./cnc_view_tui.toml`
- `~/.config/cnc_view_tui/config.toml`

Example:

```toml
[projection]
mode = "perspective"
yaw_deg = -45.0
pitch_deg = 70.0

[parser]
ignore_unknown_words = true
ignore_missing_words = ["E"]

[animation]
speed_segments_per_sec = 800.0

[ui]
show_line_numbers = false

[theme]
background = "#1e1e2e"
foreground = "#cdd6f4"
path_feed = "#89b4fa"
path_rapid = "#6c7086"
axis_x = "#f38ba8"
axis_y = "#a6e3a1"
axis_z = "#89b4fa"
grid = "#45475a"
status_fg = "#cdd6f4"
status_bg = "#313244"
code_keyword = "#cba6f7"
code_number = "#fab387"
code_comment = "#6c7086"
code_label = "#f9e2af"
code_axis = "#94e2d5"
```
