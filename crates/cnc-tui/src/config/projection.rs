use anyhow::{anyhow, Result};
use serde::Deserialize;

use cnc_geom::ProjectionMode;

#[derive(Debug, Clone)]
pub struct ProjectionSettings {
    pub mode: ProjectionMode,
    pub yaw_deg: f64,
    pub pitch_deg: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct ProjectionConfig {
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
