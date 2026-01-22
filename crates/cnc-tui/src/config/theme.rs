use anyhow::Result;
use ratatui::style::Color;
use serde::Deserialize;

use super::parse::parse_color;

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

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct ThemeConfig {
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
