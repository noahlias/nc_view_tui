use anyhow::{anyhow, Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use cnc_geom::ProjectionMode;

#[derive(Clone)]
pub struct Config {
    pub keys: KeyBindings,
    pub theme: Theme,
    pub projection: ProjectionSettings,
    pub parser: ParserSettings,
    pub animation: AnimationSettings,
    pub ui: UiSettings,
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> Result<Self> {
        let cfg_path = match path {
            Some(p) => Some(p),
            None => find_default_config(),
        };

        let file_cfg = match cfg_path {
            Some(ref p) if p.exists() => {
                let raw = fs::read_to_string(p)
                    .with_context(|| format!("failed to read config: {}", p.display()))?;
                toml::from_str::<FileConfig>(&raw)
                    .with_context(|| format!("failed to parse config: {}", p.display()))?
            }
            Some(p) => {
                return Err(anyhow!("config file not found: {}", p.display()));
            }
            None => FileConfig::default(),
        };

        file_cfg.try_into()
    }
}

fn find_default_config() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok();
    if let Some(dir) = cwd {
        let candidate = dir.join("cnc_view_tui.toml");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    let home = std::env::var("HOME").ok();
    if let Some(home_dir) = home {
        let candidate = Path::new(&home_dir)
            .join(".config")
            .join("cnc_view_tui")
            .join("config.toml");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

#[derive(Debug, Clone)]
pub struct ProjectionSettings {
    pub mode: ProjectionMode,
    pub yaw_deg: f64,
    pub pitch_deg: f64,
}

#[derive(Debug, Clone)]
pub struct ParserSettings {
    pub ignore_missing_words: Vec<char>,
    pub ignore_unknown_words: bool,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub path_feed: Color,
    pub path_rapid: Color,
    pub axis_x: Color,
    pub axis_y: Color,
    pub axis_z: Color,
    pub grid: Color,
    pub status_fg: Color,
    pub status_bg: Color,
    pub code_keyword: Color,
    pub code_number: Color,
    pub code_comment: Color,
    pub code_label: Color,
    pub code_axis: Color,
}

#[derive(Debug, Clone)]
pub struct KeyBindings {
    pub quit: KeySpec,
    pub pan_left: KeySpec,
    pub pan_right: KeySpec,
    pub pan_up: KeySpec,
    pub pan_down: KeySpec,
    pub zoom_in: KeySpec,
    pub zoom_out: KeySpec,
    pub rotate_left: KeySpec,
    pub rotate_right: KeySpec,
    pub rotate_up: KeySpec,
    pub rotate_down: KeySpec,
    pub fit: KeySpec,
    pub reset_view: KeySpec,
    pub toggle_playback: KeySpec,
    pub toggle_focus: KeySpec,
    pub line_up: KeySpec,
    pub line_down: KeySpec,
    pub page_up: KeySpec,
    pub page_down: KeySpec,
    pub toggle_projection: KeySpec,
    pub toggle_help: KeySpec,
    pub toggle_visual: KeySpec,
    pub toggle_marker: KeySpec,
}

impl KeyBindings {
    pub fn action_for(&self, key: KeyEvent) -> Option<Action> {
        if self.quit.matches(key) {
            return Some(Action::Quit);
        }
        if self.pan_left.matches(key) {
            return Some(Action::PanLeft);
        }
        if self.pan_right.matches(key) {
            return Some(Action::PanRight);
        }
        if self.pan_up.matches(key) {
            return Some(Action::PanUp);
        }
        if self.pan_down.matches(key) {
            return Some(Action::PanDown);
        }
        if self.zoom_in.matches(key) {
            return Some(Action::ZoomIn);
        }
        if self.zoom_out.matches(key) {
            return Some(Action::ZoomOut);
        }
        if self.rotate_left.matches(key) {
            return Some(Action::RotateLeft);
        }
        if self.rotate_right.matches(key) {
            return Some(Action::RotateRight);
        }
        if self.rotate_up.matches(key) {
            return Some(Action::RotateUp);
        }
        if self.rotate_down.matches(key) {
            return Some(Action::RotateDown);
        }
        if self.fit.matches(key) {
            return Some(Action::Fit);
        }
        if self.reset_view.matches(key) {
            return Some(Action::ResetView);
        }
        if self.toggle_playback.matches(key) {
            return Some(Action::TogglePlayback);
        }
        if self.toggle_focus.matches(key) {
            return Some(Action::ToggleFocus);
        }
        if self.line_up.matches(key) {
            return Some(Action::LineUp);
        }
        if self.line_down.matches(key) {
            return Some(Action::LineDown);
        }
        if self.page_up.matches(key) {
            return Some(Action::PageUp);
        }
        if self.page_down.matches(key) {
            return Some(Action::PageDown);
        }
        if self.toggle_projection.matches(key) {
            return Some(Action::ToggleProjection);
        }
        if self.toggle_help.matches(key) {
            return Some(Action::ToggleHelp);
        }
        if self.toggle_visual.matches(key) {
            return Some(Action::ToggleVisual);
        }
        if self.toggle_marker.matches(key) {
            return Some(Action::ToggleMarker);
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    PanLeft,
    PanRight,
    PanUp,
    PanDown,
    ZoomIn,
    ZoomOut,
    RotateLeft,
    RotateRight,
    RotateUp,
    RotateDown,
    Fit,
    ResetView,
    TogglePlayback,
    ToggleFocus,
    LineUp,
    LineDown,
    PageUp,
    PageDown,
    ToggleProjection,
    ToggleHelp,
    ToggleVisual,
    ToggleMarker,
}

#[derive(Debug, Clone)]
pub struct KeySpec {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeySpec {
    pub fn matches(&self, key: KeyEvent) -> bool {
        self.code == key.code && self.modifiers == key.modifiers
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct FileConfig {
    keys: KeysConfig,
    theme: ThemeConfig,
    projection: ProjectionConfig,
    parser: ParserConfig,
    animation: AnimationConfig,
    ui: UiConfig,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            keys: KeysConfig::default(),
            theme: ThemeConfig::default(),
            projection: ProjectionConfig::default(),
            parser: ParserConfig::default(),
            animation: AnimationConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl TryFrom<FileConfig> for Config {
    type Error = anyhow::Error;

    fn try_from(value: FileConfig) -> Result<Self> {
        let keys = value.keys.try_into()?;
        let theme = value.theme.try_into()?;
        let projection = value.projection.try_into()?;
        let parser = value.parser.try_into()?;
        let animation = value.animation.try_into()?;
        let ui = value.ui.try_into()?;
        Ok(Self {
            keys,
            theme,
            projection,
            parser,
            animation,
            ui,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct KeysConfig {
    quit: String,
    pan_left: String,
    pan_right: String,
    pan_up: String,
    pan_down: String,
    zoom_in: String,
    zoom_out: String,
    rotate_left: String,
    rotate_right: String,
    rotate_up: String,
    rotate_down: String,
    fit: String,
    reset_view: String,
    toggle_playback: String,
    toggle_focus: String,
    line_up: String,
    line_down: String,
    page_up: String,
    page_down: String,
    toggle_projection: String,
    toggle_help: String,
    toggle_visual: String,
    toggle_marker: String,
}

impl Default for KeysConfig {
    fn default() -> Self {
        Self {
            quit: "q".to_string(),
            pan_left: "h".to_string(),
            pan_right: "l".to_string(),
            pan_up: "k".to_string(),
            pan_down: "j".to_string(),
            zoom_in: "plus".to_string(),
            zoom_out: "minus".to_string(),
            rotate_left: "a".to_string(),
            rotate_right: "d".to_string(),
            rotate_up: "w".to_string(),
            rotate_down: "s".to_string(),
            fit: "g".to_string(),
            reset_view: "r".to_string(),
            toggle_playback: "space".to_string(),
            toggle_focus: "tab".to_string(),
            line_up: "up".to_string(),
            line_down: "down".to_string(),
            page_up: "pageup".to_string(),
            page_down: "pagedown".to_string(),
            toggle_projection: "p".to_string(),
            toggle_help: "?".to_string(),
            toggle_visual: "v".to_string(),
            toggle_marker: "m".to_string(),
        }
    }
}

impl TryFrom<KeysConfig> for KeyBindings {
    type Error = anyhow::Error;

    fn try_from(value: KeysConfig) -> Result<Self> {
        Ok(Self {
            quit: parse_key_spec(&value.quit)?,
            pan_left: parse_key_spec(&value.pan_left)?,
            pan_right: parse_key_spec(&value.pan_right)?,
            pan_up: parse_key_spec(&value.pan_up)?,
            pan_down: parse_key_spec(&value.pan_down)?,
            zoom_in: parse_key_spec(&value.zoom_in)?,
            zoom_out: parse_key_spec(&value.zoom_out)?,
            rotate_left: parse_key_spec(&value.rotate_left)?,
            rotate_right: parse_key_spec(&value.rotate_right)?,
            rotate_up: parse_key_spec(&value.rotate_up)?,
            rotate_down: parse_key_spec(&value.rotate_down)?,
            fit: parse_key_spec(&value.fit)?,
            reset_view: parse_key_spec(&value.reset_view)?,
            toggle_playback: parse_key_spec(&value.toggle_playback)?,
            toggle_focus: parse_key_spec(&value.toggle_focus)?,
            line_up: parse_key_spec(&value.line_up)?,
            line_down: parse_key_spec(&value.line_down)?,
            page_up: parse_key_spec(&value.page_up)?,
            page_down: parse_key_spec(&value.page_down)?,
            toggle_projection: parse_key_spec(&value.toggle_projection)?,
            toggle_help: parse_key_spec(&value.toggle_help)?,
            toggle_visual: parse_key_spec(&value.toggle_visual)?,
            toggle_marker: parse_key_spec(&value.toggle_marker)?,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct ThemeConfig {
    background: String,
    foreground: String,
    path_feed: String,
    path_rapid: String,
    axis_x: String,
    axis_y: String,
    axis_z: String,
    grid: String,
    status_fg: String,
    status_bg: String,
    code_keyword: String,
    code_number: String,
    code_comment: String,
    code_label: String,
    code_axis: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background: "#1e1e2e".to_string(),
            foreground: "#cdd6f4".to_string(),
            path_feed: "#89b4fa".to_string(),
            path_rapid: "#6c7086".to_string(),
            axis_x: "#f38ba8".to_string(),
            axis_y: "#a6e3a1".to_string(),
            axis_z: "#89b4fa".to_string(),
            grid: "#45475a".to_string(),
            status_fg: "#cdd6f4".to_string(),
            status_bg: "#313244".to_string(),
            code_keyword: "#cba6f7".to_string(),
            code_number: "#fab387".to_string(),
            code_comment: "#6c7086".to_string(),
            code_label: "#f9e2af".to_string(),
            code_axis: "#94e2d5".to_string(),
        }
    }
}

impl TryFrom<ThemeConfig> for Theme {
    type Error = anyhow::Error;

    fn try_from(value: ThemeConfig) -> Result<Self> {
        Ok(Self {
            background: parse_color(&value.background)?,
            foreground: parse_color(&value.foreground)?,
            path_feed: parse_color(&value.path_feed)?,
            path_rapid: parse_color(&value.path_rapid)?,
            axis_x: parse_color(&value.axis_x)?,
            axis_y: parse_color(&value.axis_y)?,
            axis_z: parse_color(&value.axis_z)?,
            grid: parse_color(&value.grid)?,
            status_fg: parse_color(&value.status_fg)?,
            status_bg: parse_color(&value.status_bg)?,
            code_keyword: parse_color(&value.code_keyword)?,
            code_number: parse_color(&value.code_number)?,
            code_comment: parse_color(&value.code_comment)?,
            code_label: parse_color(&value.code_label)?,
            code_axis: parse_color(&value.code_axis)?,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct ProjectionConfig {
    mode: String,
    yaw_deg: f64,
    pitch_deg: f64,
}

impl Default for ProjectionConfig {
    fn default() -> Self {
        Self {
            mode: "perspective".to_string(),
            yaw_deg: -45.0,
            pitch_deg: 70.0,
        }
    }
}

impl TryFrom<ProjectionConfig> for ProjectionSettings {
    type Error = anyhow::Error;

    fn try_from(value: ProjectionConfig) -> Result<Self> {
        let mode = value
            .mode
            .parse::<ProjectionMode>()
            .map_err(|err| anyhow!(err))?;
        Ok(Self {
            mode,
            yaw_deg: value.yaw_deg,
            pitch_deg: value.pitch_deg,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct ParserConfig {
    ignore_missing_words: Vec<String>,
    ignore_unknown_words: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            ignore_missing_words: vec!["E".to_string()],
            ignore_unknown_words: true,
        }
    }
}

impl TryFrom<ParserConfig> for ParserSettings {
    type Error = anyhow::Error;

    fn try_from(value: ParserConfig) -> Result<Self> {
        let mut ignore_missing_words = Vec::new();
        for item in value.ignore_missing_words {
            let trimmed = item.trim();
            if trimmed.is_empty() {
                continue;
            }
            let mut chars = trimmed.chars();
            let ch = chars
                .next()
                .ok_or_else(|| anyhow!("empty ignore_missing_words entry"))?;
            if chars.next().is_some() {
                return Err(anyhow!(
                    "ignore_missing_words entry must be a single letter: {}",
                    item
                ));
            }
            ignore_missing_words.push(ch.to_ascii_uppercase());
        }
        Ok(Self {
            ignore_missing_words,
            ignore_unknown_words: value.ignore_unknown_words,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AnimationSettings {
    pub speed_segments_per_sec: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct AnimationConfig {
    speed_segments_per_sec: f64,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            speed_segments_per_sec: 800.0,
        }
    }
}

impl TryFrom<AnimationConfig> for AnimationSettings {
    type Error = anyhow::Error;

    fn try_from(value: AnimationConfig) -> Result<Self> {
        if value.speed_segments_per_sec <= 0.0 {
            return Err(anyhow!("animation speed must be positive"));
        }
        Ok(Self {
            speed_segments_per_sec: value.speed_segments_per_sec,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UiSettings {
    pub show_line_numbers: bool,
    pub canvas_marker: ratatui::symbols::Marker,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct UiConfig {
    show_line_numbers: bool,
    canvas_marker: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_line_numbers: false,
            canvas_marker: "braille".to_string(),
        }
    }
}

impl TryFrom<UiConfig> for UiSettings {
    type Error = anyhow::Error;

    fn try_from(value: UiConfig) -> Result<Self> {
        let canvas_marker = parse_marker(&value.canvas_marker)?;
        Ok(Self {
            show_line_numbers: value.show_line_numbers,
            canvas_marker,
        })
    }
}

fn parse_marker(raw: &str) -> Result<ratatui::symbols::Marker> {
    let value = raw.trim().to_ascii_lowercase();
    match value.as_str() {
        "braille" => Ok(ratatui::symbols::Marker::Braille),
        "halfblock" | "half_block" | "half-block" => Ok(ratatui::symbols::Marker::HalfBlock),
        "dot" => Ok(ratatui::symbols::Marker::Dot),
        "block" => Ok(ratatui::symbols::Marker::Block),
        "bar" => Ok(ratatui::symbols::Marker::Bar),
        _ => Err(anyhow!("unknown canvas_marker: {}", raw)),
    }
}

fn parse_key_spec(raw: &str) -> Result<KeySpec> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("empty key binding"));
    }
    if trimmed.len() == 1 {
        let ch = trimmed.chars().next().unwrap();
        return Ok(KeySpec {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::empty(),
        });
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower == "plus" {
        return Ok(KeySpec {
            code: KeyCode::Char('+'),
            modifiers: KeyModifiers::empty(),
        });
    }
    if lower == "minus" {
        return Ok(KeySpec {
            code: KeyCode::Char('-'),
            modifiers: KeyModifiers::empty(),
        });
    }
    if lower == "space" {
        return Ok(KeySpec {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::empty(),
        });
    }

    let mut modifiers = KeyModifiers::empty();
    let parts: Vec<&str> = lower.split('+').collect();
    if parts.is_empty() {
        return Err(anyhow!("invalid key binding: {}", raw));
    }

    let key_part = parts.last().unwrap().trim();
    for part in &parts[..parts.len().saturating_sub(1)] {
        match part.trim() {
            "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
            "alt" => modifiers |= KeyModifiers::ALT,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            "" => {}
            other => return Err(anyhow!("unknown modifier: {}", other)),
        }
    }

    let code = parse_key_code(key_part)?;
    Ok(KeySpec { code, modifiers })
}

fn parse_key_code(raw: &str) -> Result<KeyCode> {
    if raw.len() == 1 {
        let ch = raw.chars().next().unwrap();
        return Ok(KeyCode::Char(ch));
    }

    match raw {
        "esc" | "escape" => Ok(KeyCode::Esc),
        "enter" | "return" => Ok(KeyCode::Enter),
        "tab" => Ok(KeyCode::Tab),
        "backspace" => Ok(KeyCode::Backspace),
        "left" => Ok(KeyCode::Left),
        "right" => Ok(KeyCode::Right),
        "up" => Ok(KeyCode::Up),
        "down" => Ok(KeyCode::Down),
        "home" => Ok(KeyCode::Home),
        "end" => Ok(KeyCode::End),
        "pageup" => Ok(KeyCode::PageUp),
        "pagedown" => Ok(KeyCode::PageDown),
        "plus" => Ok(KeyCode::Char('+')),
        "minus" => Ok(KeyCode::Char('-')),
        other => Err(anyhow!("unknown key: {}", other)),
    }
}

fn parse_color(raw: &str) -> Result<Color> {
    let lower = raw.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return Err(anyhow!("empty color"));
    }
    if let Some(color) = parse_named_color(&lower) {
        return Ok(color);
    }
    if let Some(color) = parse_hex_color(&lower) {
        return Ok(color);
    }
    Err(anyhow!("unknown color: {}", raw))
}

fn parse_named_color(name: &str) -> Option<Color> {
    match name {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "lightred" => Some(Color::LightRed),
        "lightgreen" => Some(Color::LightGreen),
        "lightyellow" => Some(Color::LightYellow),
        "lightblue" => Some(Color::LightBlue),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightcyan" => Some(Color::LightCyan),
        "white" => Some(Color::White),
        _ => None,
    }
}

fn parse_hex_color(raw: &str) -> Option<Color> {
    let hex = raw.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_marker_values() {
        assert!(matches!(parse_marker("braille").unwrap(), ratatui::symbols::Marker::Braille));
        assert!(matches!(
            parse_marker("halfblock").unwrap(),
            ratatui::symbols::Marker::HalfBlock
        ));
        assert!(matches!(parse_marker("dot").unwrap(), ratatui::symbols::Marker::Dot));
        assert!(matches!(parse_marker("block").unwrap(), ratatui::symbols::Marker::Block));
        assert!(matches!(parse_marker("bar").unwrap(), ratatui::symbols::Marker::Bar));
    }
}
