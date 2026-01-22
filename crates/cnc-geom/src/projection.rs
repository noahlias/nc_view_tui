use crate::geom::{Vec2, Vec3};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionMode {
    Orthographic,
    Perspective,
}

impl FromStr for ProjectionMode {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "orthographic" | "ortho" => Ok(Self::Orthographic),
            "perspective" | "persp" => Ok(Self::Perspective),
            other => Err(format!("unknown projection mode: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ViewAngles {
    pub yaw: f64,
    pub pitch: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectionParams {
    pub mode: ProjectionMode,
    pub angles: ViewAngles,
    pub camera_distance: f64,
    pub target: Vec3,
}

pub fn project_point(p: Vec3, params: ProjectionParams) -> Vec2 {
    let translated = p - params.target;
    let rotated = rotate_point(translated, params.angles);
    match params.mode {
        ProjectionMode::Orthographic => Vec2::new(rotated.x, rotated.y),
        ProjectionMode::Perspective => {
            let denom = params.camera_distance + rotated.z;
            let factor = if denom.abs() < 1e-6 {
                1.0
            } else {
                params.camera_distance / denom
            };
            Vec2::new(rotated.x * factor, rotated.y * factor)
        }
    }
}

fn rotate_point(p: Vec3, angles: ViewAngles) -> Vec3 {
    let (sy, cy) = angles.yaw.sin_cos();
    let (sp, cp) = angles.pitch.sin_cos();

    let x1 = p.x * cy - p.y * sy;
    let y1 = p.x * sy + p.y * cy;
    let z1 = p.z;

    let y2 = y1 * cp - z1 * sp;
    let z2 = y1 * sp + z1 * cp;

    Vec3::new(x1, y2, z2)
}
