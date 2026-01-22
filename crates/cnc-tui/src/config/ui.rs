use anyhow::Result;
use serde::Deserialize;

use super::parse::parse_marker;

#[derive(Debug, Clone)]
pub struct UiSettings {
    pub show_line_numbers: bool,
    pub canvas_marker: ratatui::symbols::Marker,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct UiConfig {
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
