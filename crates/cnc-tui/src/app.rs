use std::path::PathBuf;

use ratatui::layout::Rect;

use crate::config::{Action, Config};
use cnc_geom::{project_point, Bounds2, Bounds3, ProjectionMode, ProjectionParams, Vec2, Vec3, ViewAngles};
use cnc_gcode::Toolpath;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ViewState {
    pub pan: Vec2,
    pub zoom: f64,
    pub yaw: f64,
    pub pitch: f64,
    pub projection: ProjectionMode,
}

#[derive(Debug, Clone, Copy)]
pub struct ViewMetrics {
    pub center: Vec2,
    pub half_w: f64,
    pub half_h: f64,
    pub camera_distance: f64,
    pub target: Vec3,
}

pub struct App {
    pub config: Config,
    pub toolpath: Toolpath,
    pub file_path: PathBuf,
    pub file_lines: Vec<String>,
    pub view: ViewState,
    pub initial_view: ViewState,
    pub last_metrics: Option<ViewMetrics>,
    pub file_panel: FilePanelState,
    pub playback: PlaybackState,
    pub show_help: bool,
}

impl App {
    pub fn new(
        config: Config,
        toolpath: Toolpath,
        file_path: PathBuf,
        file_lines: Vec<String>,
    ) -> Self {
        let view = ViewState {
            pan: Vec2::new(0.0, 0.0),
            zoom: 1.0,
            yaw: config.projection.yaw_deg.to_radians(),
            pitch: config.projection.pitch_deg.to_radians(),
            projection: config.projection.mode,
        };
        let playback = PlaybackState::new(config.animation.speed_segments_per_sec);
        let file_panel = FilePanelState::new(file_lines.len());
        Self {
            config,
            toolpath,
            file_path,
            file_lines,
            view: view.clone(),
            initial_view: view,
            last_metrics: None,
            file_panel,
            playback,
            show_help: false,
        }
    }

    pub fn apply_action(&mut self, action: Action) {
        if self.show_help {
            match action {
                Action::ToggleHelp | Action::Quit => {
                    self.show_help = false;
                }
                _ => {}
            }
            return;
        }
        match action {
            Action::PanLeft => self.apply_view_pan(-1.0, 0.0),
            Action::PanRight => self.apply_view_pan(1.0, 0.0),
            Action::PanUp => self.apply_view_pan(0.0, 1.0),
            Action::PanDown => self.apply_view_pan(0.0, -1.0),
            Action::ZoomIn => self.view.zoom *= 1.1,
            Action::ZoomOut => self.view.zoom = (self.view.zoom / 1.1).max(0.05),
            Action::RotateLeft => self.view.yaw -= 5.0_f64.to_radians(),
            Action::RotateRight => self.view.yaw += 5.0_f64.to_radians(),
            Action::RotateUp => self.view.pitch += 5.0_f64.to_radians(),
            Action::RotateDown => self.view.pitch -= 5.0_f64.to_radians(),
            Action::Fit => {
                self.view.pan = Vec2::new(0.0, 0.0);
                self.view.zoom = 1.0;
            }
            Action::ResetView => {
                self.view = self.initial_view.clone();
            }
            Action::ToggleProjection => {
                self.view.projection = match self.view.projection {
                    ProjectionMode::Orthographic => ProjectionMode::Perspective,
                    ProjectionMode::Perspective => ProjectionMode::Orthographic,
                }
            }
            Action::TogglePlayback => {
                let total = self.toolpath.segments.len();
                self.playback.toggle(total);
            }
            Action::ToggleHelp => {
                self.show_help = !self.show_help;
            }
            Action::ToggleVisual => {
                if self.file_panel.focus == PanelFocus::File {
                    self.file_panel.toggle_visual();
                }
            }
            Action::ToggleFocus => self.file_panel.toggle_focus(),
            Action::LineUp => {
                if self.file_panel.focus == PanelFocus::File {
                    self.file_panel.move_selection(-1, self.file_lines.len());
                }
            }
            Action::LineDown => {
                if self.file_panel.focus == PanelFocus::File {
                    self.file_panel.move_selection(1, self.file_lines.len());
                }
            }
            Action::PageUp => {
                if self.file_panel.focus == PanelFocus::File {
                    self.file_panel.page_selection(-1, self.file_lines.len());
                }
            }
            Action::PageDown => {
                if self.file_panel.focus == PanelFocus::File {
                    self.file_panel.page_selection(1, self.file_lines.len());
                }
            }
            Action::Quit => {}
        }
    }

    fn pan_step(&self) -> (f64, f64) {
        if let Some(metrics) = self.last_metrics {
            let step_x = (metrics.half_w * 0.1).max(0.1);
            let step_y = (metrics.half_h * 0.1).max(0.1);
            (step_x, step_y)
        } else {
            (1.0, 1.0)
        }
    }

    fn apply_view_pan(&mut self, dx: f64, dy: f64) {
        let (step_x, step_y) = self.pan_step();
        self.view.pan.x += step_x * dx;
        self.view.pan.y += step_y * dy;
    }

    pub fn compute_view_metrics(&self, area: Rect) -> ViewMetrics {
        let (base_bounds, camera_distance, target) = self.projected_bounds();
        let mut half_w = (base_bounds.width() * 0.5) / self.view.zoom;
        let mut half_h = (base_bounds.height() * 0.5) / self.view.zoom;
        if half_w < 1e-6 {
            half_w = 1.0;
        }
        if half_h < 1e-6 {
            half_h = 1.0;
        }

        let aspect = if area.height == 0 {
            1.0
        } else {
            area.width as f64 / area.height as f64
        };
        if aspect.is_finite() {
            if (half_w / half_h) > aspect {
                half_h = half_w / aspect;
            } else {
                half_w = half_h * aspect;
            }
        }

        let center = base_bounds.center() + self.view.pan;
        ViewMetrics {
            center,
            half_w,
            half_h,
            camera_distance,
            target,
        }
    }

    fn projected_bounds(&self) -> (Bounds2, f64, Vec3) {
        let mut bounds = Bounds2::new();
        if !self.toolpath.bounds.initialized {
            let mut default_bounds = Bounds2::new();
            default_bounds.include(Vec2::new(-1.0, -1.0));
            default_bounds.include(Vec2::new(1.0, 1.0));
            return (default_bounds, 10.0, Vec3::new(0.0, 0.0, 0.0));
        }

        let size = self.toolpath.bounds.size();
        let max_dim = size.x.max(size.y).max(size.z).max(1.0);
        let camera_distance = max_dim * 2.5;
        let target = self.toolpath.bounds.center();
        let params = ProjectionParams {
            mode: self.view.projection,
            angles: ViewAngles {
                yaw: self.view.yaw,
                pitch: self.view.pitch,
            },
            camera_distance,
            target,
        };

        for corner in bounds_corners(self.toolpath.bounds) {
            let p = project_point(corner, params);
            bounds.include(p);
        }

        if !bounds.initialized {
            bounds.include(Vec2::new(-1.0, -1.0));
            bounds.include(Vec2::new(1.0, 1.0));
        }

        (bounds, camera_distance, target)
    }

    pub fn tick(&mut self, delta: Duration) {
        self.playback
            .tick(delta, self.toolpath.segments.len());
    }

    pub fn visible_segment_count(&self) -> usize {
        let total = self.toolpath.segments.len();
        if total == 0 {
            return 0;
        }
        let (start, end) = self.visible_segment_range();
        end.saturating_sub(start)
    }

    pub fn visible_segment_range(&self) -> (usize, usize) {
        let total = self.toolpath.segments.len();
        if total == 0 {
            return (0, 0);
        }
        let ends = &self.toolpath.line_segment_ends;
        if ends.is_empty() {
            return (0, total);
        }
        let (line_start, line_end) = self.file_panel.selection_range(self.file_lines.len());
        let max_line = ends.len().saturating_sub(1);
        let start_line = line_start.min(max_line);
        let end_line = line_end.min(max_line);
        let start_idx = if start_line == 0 { 0 } else { ends[start_line - 1] };
        let mut end_idx = ends[end_line].min(total);
        if end_idx < start_idx {
            end_idx = start_idx;
        }
        let range_len = end_idx - start_idx;
        let visible_len = if self.playback.active {
            self.playback.visible_segments(range_len)
        } else {
            range_len
        };
        (start_idx, start_idx + visible_len)
    }
}

fn bounds_corners(bounds: Bounds3) -> [Vec3; 8] {
    let min = bounds.min;
    let max = bounds.max;
    [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(min.x, max.y, max.z),
        Vec3::new(max.x, max.y, max.z),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    Viewport,
    File,
}

#[derive(Debug, Clone)]
pub struct FilePanelState {
    pub focus: PanelFocus,
    pub selected: usize,
    pub scroll: usize,
    pub view_height: usize,
    pub visual: bool,
    pub anchor: usize,
}

impl FilePanelState {
    pub fn new(total_lines: usize) -> Self {
        let selected = total_lines.saturating_sub(1);
        Self {
            focus: PanelFocus::Viewport,
            selected,
            scroll: 0,
            view_height: 0,
            visual: total_lines > 0,
            anchor: 0,
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            PanelFocus::Viewport => PanelFocus::File,
            PanelFocus::File => PanelFocus::Viewport,
        };
    }

    pub fn toggle_visual(&mut self) {
        if self.visual {
            self.visual = false;
        } else {
            self.visual = true;
            self.anchor = self.selected;
        }
    }

    pub fn move_selection(&mut self, delta: isize, total: usize) {
        if total == 0 {
            self.selected = 0;
            self.scroll = 0;
            return;
        }
        let next = self.selected as isize + delta;
        self.selected = next.clamp(0, (total - 1) as isize) as usize;
        self.ensure_visible();
    }

    pub fn page_selection(&mut self, direction: isize, total: usize) {
        let page = self.view_height.max(1) as isize;
        self.move_selection(direction * page, total);
    }

    pub fn ensure_visible(&mut self) {
        if self.view_height == 0 {
            return;
        }
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + self.view_height {
            self.scroll = self.selected + 1 - self.view_height;
        }
    }

    pub fn selection_range(&self, total: usize) -> (usize, usize) {
        if total == 0 {
            return (0, 0);
        }
        if self.visual {
            let start = self.anchor.min(self.selected);
            let end = self.anchor.max(self.selected);
            (start, end)
        } else {
            (self.selected, self.selected)
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlaybackState {
    pub active: bool,
    pub playing: bool,
    pub position: f64,
    pub speed: f64,
}

impl PlaybackState {
    pub fn new(speed: f64) -> Self {
        Self {
            active: false,
            playing: false,
            position: 0.0,
            speed,
        }
    }

    pub fn toggle(&mut self, total: usize) {
        if !self.active {
            self.active = true;
            self.playing = true;
            if self.position >= total as f64 {
                self.position = 0.0;
            }
            return;
        }

        if self.playing {
            self.playing = false;
        } else {
            if self.position >= total as f64 {
                self.position = 0.0;
            }
            self.playing = true;
        }
    }

    pub fn tick(&mut self, delta: Duration, total: usize) {
        if !self.playing {
            return;
        }
        let add = delta.as_secs_f64() * self.speed;
        self.position += add;
        if self.position >= total as f64 {
            self.position = total as f64;
            self.playing = false;
            self.active = false;
        }
    }

    pub fn visible_segments(&self, total: usize) -> usize {
        if !self.active {
            return total;
        }
        (self.position.floor() as usize).min(total)
    }
}
