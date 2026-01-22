use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::Deserialize;

use super::parse::parse_key_spec;

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
pub(crate) struct KeysConfig {
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
