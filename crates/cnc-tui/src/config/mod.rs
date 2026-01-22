use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

mod animation;
mod keys;
mod parse;
mod parser;
mod projection;
mod theme;
mod ui;

pub use animation::AnimationSettings;
pub use keys::{Action, KeyBindings, KeySpec};
pub use parser::ParserSettings;
pub use projection::ProjectionSettings;
pub use theme::Theme;
pub use ui::UiSettings;

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

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct FileConfig {
    keys: keys::KeysConfig,
    theme: theme::ThemeConfig,
    projection: projection::ProjectionConfig,
    parser: parser::ParserConfig,
    animation: animation::AnimationConfig,
    ui: ui::UiConfig,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            keys: keys::KeysConfig::default(),
            theme: theme::ThemeConfig::default(),
            projection: projection::ProjectionConfig::default(),
            parser: parser::ParserConfig::default(),
            animation: animation::AnimationConfig::default(),
            ui: ui::UiConfig::default(),
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
