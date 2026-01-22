use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct AnimationSettings {
    pub speed_segments_per_sec: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct AnimationConfig {
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
