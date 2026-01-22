use anyhow::{anyhow, Context, Result};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use cnc_geom::{Bounds3, Vec3};

const ARC_SEGMENT_LENGTH: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct ParseOptions {
    ignore_missing_value: HashSet<char>,
    ignore_unknown_words: bool,
}

impl ParseOptions {
    pub fn with_ignore_missing<I>(letters: I) -> Self
    where
        I: IntoIterator<Item = char>,
    {
        let ignore_missing_value = letters
            .into_iter()
            .map(|c| c.to_ascii_uppercase())
            .collect();
        Self {
            ignore_missing_value,
            ignore_unknown_words: false,
        }
    }

    pub fn with_ignore_unknown_words(mut self, ignore_unknown_words: bool) -> Self {
        self.ignore_unknown_words = ignore_unknown_words;
        self
    }

    fn should_ignore_missing(&self, letter: char) -> bool {
        self.ignore_missing_value.contains(&letter)
    }
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            ignore_missing_value: HashSet::new(),
            ignore_unknown_words: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MoveKind {
    Rapid,
    Feed,
}

#[derive(Debug, Clone, Copy)]
pub struct LineSegment {
    pub start: Vec3,
    pub end: Vec3,
    pub kind: MoveKind,
}

#[derive(Debug, Default, Clone)]
pub struct ToolpathStats {
    pub line_count: usize,
    pub segment_count: usize,
    pub rapid_moves: usize,
    pub feed_moves: usize,
    pub arc_moves: usize,
}

#[derive(Debug, Clone)]
pub struct Toolpath {
    pub segments: Vec<LineSegment>,
    pub bounds: Bounds3,
    pub stats: ToolpathStats,
    pub line_segment_ends: Vec<usize>,
}

#[derive(Debug, Clone, Copy)]
enum DistanceMode {
    Absolute,
    Relative,
}

#[derive(Debug, Clone, Copy)]
enum Plane {
    XY,
    XZ,
    YZ,
}

#[derive(Debug, Clone, Copy)]
enum MotionMode {
    Rapid,
    Feed,
    ArcCW,
    ArcCCW,
}

#[derive(Debug, Clone, Copy)]
struct ParserState {
    pos: Vec3,
    units_scale: f64,
    distance_mode: DistanceMode,
    plane: Plane,
    motion_mode: MotionMode,
}

impl ParserState {
    fn new() -> Self {
        Self {
            pos: Vec3::new(0.0, 0.0, 0.0),
            units_scale: 1.0,
            distance_mode: DistanceMode::Absolute,
            plane: Plane::XY,
            motion_mode: MotionMode::Rapid,
        }
    }
}

pub fn parse_file(path: &Path) -> Result<Toolpath> {
    parse_file_with_options(path, ParseOptions::default())
}

pub fn parse_file_with_options(path: &Path, options: ParseOptions) -> Result<Toolpath> {
    let file = File::open(path)
        .with_context(|| format!("failed to open g-code: {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut parser = Parser::new(options);

    for (idx, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("failed to read line {}", idx + 1))?;
        parser.parse_line(&line, idx + 1)?;
    }

    parser.finish()
}

struct Parser {
    state: ParserState,
    segments: Vec<LineSegment>,
    bounds: Bounds3,
    stats: ToolpathStats,
    options: ParseOptions,
    line_segment_ends: Vec<usize>,
}

impl Parser {
    fn new(options: ParseOptions) -> Self {
        Self {
            state: ParserState::new(),
            segments: Vec::new(),
            bounds: Bounds3::new(),
            stats: ToolpathStats::default(),
            options,
            line_segment_ends: Vec::new(),
        }
    }

    fn finish(mut self) -> Result<Toolpath> {
        self.stats.segment_count = self.segments.len();
        Ok(Toolpath {
            segments: self.segments,
            bounds: self.bounds,
            stats: self.stats,
            line_segment_ends: self.line_segment_ends,
        })
    }

    fn parse_line(&mut self, line: &str, line_no: usize) -> Result<()> {
        self.stats.line_count += 1;
        let cleaned = strip_comments(line);
        let cleaned = cleaned.trim();
        if cleaned.is_empty() {
            self.line_segment_ends.push(self.segments.len());
            return Ok(());
        }

        let words =
            parse_words(cleaned, &self.options).with_context(|| format!("line {}", line_no))?;
        if words.is_empty() {
            self.line_segment_ends.push(self.segments.len());
            return Ok(());
        }

        let mut motion_override: Option<MotionMode> = None;
        let mut x: Option<f64> = None;
        let mut y: Option<f64> = None;
        let mut z: Option<f64> = None;
        let mut i: Option<f64> = None;
        let mut j: Option<f64> = None;
        let mut k: Option<f64> = None;
        let mut r: Option<f64> = None;

        for word in words {
            match word.letter {
                'G' => {
                    let code = word.value.round() as i32;
                    match code {
                        0 => {
                            motion_override = Some(MotionMode::Rapid);
                            self.state.motion_mode = MotionMode::Rapid;
                        }
                        1 => {
                            motion_override = Some(MotionMode::Feed);
                            self.state.motion_mode = MotionMode::Feed;
                        }
                        2 => {
                            motion_override = Some(MotionMode::ArcCW);
                            self.state.motion_mode = MotionMode::ArcCW;
                        }
                        3 => {
                            motion_override = Some(MotionMode::ArcCCW);
                            self.state.motion_mode = MotionMode::ArcCCW;
                        }
                        17 => self.state.plane = Plane::XY,
                        18 => self.state.plane = Plane::XZ,
                        19 => self.state.plane = Plane::YZ,
                        20 => self.state.units_scale = 25.4,
                        21 => self.state.units_scale = 1.0,
                        90 => self.state.distance_mode = DistanceMode::Absolute,
                        91 => self.state.distance_mode = DistanceMode::Relative,
                        _ => {}
                    }
                }
                'X' => x = Some(word.value * self.state.units_scale),
                'Y' => y = Some(word.value * self.state.units_scale),
                'Z' => z = Some(word.value * self.state.units_scale),
                'I' => i = Some(word.value * self.state.units_scale),
                'J' => j = Some(word.value * self.state.units_scale),
                'K' => k = Some(word.value * self.state.units_scale),
                'R' => r = Some(word.value * self.state.units_scale),
                _ => {}
            }
        }

        let motion = if motion_override.is_some() {
            motion_override
        } else if x.is_some() || y.is_some() || z.is_some() || i.is_some() || j.is_some() || k.is_some() {
            Some(self.state.motion_mode)
        } else {
            None
        };

        if let Some(mode) = motion {
            match mode {
                MotionMode::Rapid => {
                    self.add_linear_move(x, y, z, MoveKind::Rapid);
                }
                MotionMode::Feed => {
                    self.add_linear_move(x, y, z, MoveKind::Feed);
                }
                MotionMode::ArcCW => {
                    self.add_arc_move(x, y, z, i, j, k, r, true)?;
                }
                MotionMode::ArcCCW => {
                    self.add_arc_move(x, y, z, i, j, k, r, false)?;
                }
            }
        }

        self.line_segment_ends.push(self.segments.len());
        Ok(())
    }

    fn add_linear_move(&mut self, x: Option<f64>, y: Option<f64>, z: Option<f64>, kind: MoveKind) {
        let start = self.state.pos;
        let mut end = start;

        apply_axis(&mut end.x, x, start.x, self.state.distance_mode);
        apply_axis(&mut end.y, y, start.y, self.state.distance_mode);
        apply_axis(&mut end.z, z, start.z, self.state.distance_mode);

        if end == start {
            return;
        }

        self.segments.push(LineSegment { start, end, kind });
        self.bounds.include(start);
        self.bounds.include(end);
        self.state.pos = end;
        match kind {
            MoveKind::Rapid => self.stats.rapid_moves += 1,
            MoveKind::Feed => self.stats.feed_moves += 1,
        }
    }

    fn add_arc_move(
        &mut self,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        i: Option<f64>,
        j: Option<f64>,
        k: Option<f64>,
        r: Option<f64>,
        clockwise: bool,
    ) -> Result<()> {
        let start = self.state.pos;
        let mut end = start;

        apply_axis(&mut end.x, x, start.x, self.state.distance_mode);
        apply_axis(&mut end.y, y, start.y, self.state.distance_mode);
        apply_axis(&mut end.z, z, start.z, self.state.distance_mode);

        if end == start {
            return Ok(());
        }

        let center = arc_center(start, end, i, j, k, r, self.state.plane, clockwise)?;
        let segments = arc_to_segments(start, end, center, clockwise, self.state.plane);

        if segments.is_empty() {
            return Ok(());
        }

        for seg in segments {
            self.segments.push(seg);
            self.bounds.include(seg.start);
            self.bounds.include(seg.end);
        }

        self.state.pos = end;
        self.stats.arc_moves += 1;
        self.stats.feed_moves += 1;
        Ok(())
    }
}

fn apply_axis(axis: &mut f64, input: Option<f64>, current: f64, mode: DistanceMode) {
    if let Some(value) = input {
        match mode {
            DistanceMode::Absolute => *axis = value,
            DistanceMode::Relative => *axis = current + value,
        }
    }
}

fn arc_center(
    start: Vec3,
    end: Vec3,
    i: Option<f64>,
    j: Option<f64>,
    k: Option<f64>,
    r: Option<f64>,
    plane: Plane,
    clockwise: bool,
) -> Result<Vec3> {
    if i.is_some() || j.is_some() || k.is_some() {
        return arc_center_from_offsets(start, i, j, k, plane);
    }
    if let Some(radius) = r {
        return arc_center_from_radius(start, end, radius, plane, clockwise);
    }
    Err(anyhow!("arc center offsets missing (IJK or R)"))
}

fn arc_center_from_offsets(
    start: Vec3,
    i: Option<f64>,
    j: Option<f64>,
    k: Option<f64>,
    plane: Plane,
) -> Result<Vec3> {
    let mut center = start;
    match plane {
        Plane::XY => {
            center.x = start.x + i.unwrap_or(0.0);
            center.y = start.y + j.unwrap_or(0.0);
        }
        Plane::XZ => {
            center.x = start.x + i.unwrap_or(0.0);
            center.z = start.z + k.unwrap_or(0.0);
        }
        Plane::YZ => {
            center.y = start.y + j.unwrap_or(0.0);
            center.z = start.z + k.unwrap_or(0.0);
        }
    }

    if center == start {
        return Err(anyhow!("arc center offsets missing"));
    }

    Ok(center)
}

fn arc_center_from_radius(
    start: Vec3,
    end: Vec3,
    radius: f64,
    plane: Plane,
    clockwise: bool,
) -> Result<Vec3> {
    let (sx, sy) = plane_coords(start, plane);
    let (ex, ey) = plane_coords(end, plane);
    let dx = ex - sx;
    let dy = ey - sy;
    let chord = (dx * dx + dy * dy).sqrt();
    if chord.abs() < 1e-9 {
        return Err(anyhow!("arc radius with coincident endpoints"));
    }

    let r_abs = radius.abs();
    if chord > 2.0 * r_abs + 1e-9 {
        return Err(anyhow!("arc radius too small for chord"));
    }

    let mid_x = (sx + ex) * 0.5;
    let mid_y = (sy + ey) * 0.5;
    let h_sq = r_abs * r_abs - (chord * 0.5).powi(2);
    let h = if h_sq <= 0.0 { 0.0 } else { h_sq.sqrt() };
    let perp_x = -dy / chord;
    let perp_y = dx / chord;

    let c1 = (mid_x + perp_x * h, mid_y + perp_y * h);
    let c2 = (mid_x - perp_x * h, mid_y - perp_y * h);

    let sweep1 = sweep_for_center(sx, sy, ex, ey, c1.0, c1.1, clockwise);
    let sweep2 = sweep_for_center(sx, sy, ex, ey, c2.0, c2.1, clockwise);
    let large = radius < 0.0;

    let pick_first = select_center(sweep1, sweep2, large);
    let (cx, cy) = if pick_first { c1 } else { c2 };

    let mut center = start;
    match plane {
        Plane::XY => {
            center.x = cx;
            center.y = cy;
        }
        Plane::XZ => {
            center.x = cx;
            center.z = cy;
        }
        Plane::YZ => {
            center.y = cx;
            center.z = cy;
        }
    }
    Ok(center)
}

fn select_center(sweep1: f64, sweep2: f64, large: bool) -> bool {
    let s1 = sweep1.abs();
    let s2 = sweep2.abs();
    if large {
        if s1 > std::f64::consts::PI && s2 <= std::f64::consts::PI {
            true
        } else if s2 > std::f64::consts::PI && s1 <= std::f64::consts::PI {
            false
        } else {
            s1 >= s2
        }
    } else if s1 <= std::f64::consts::PI && s2 > std::f64::consts::PI {
        true
    } else if s2 <= std::f64::consts::PI && s1 > std::f64::consts::PI {
        false
    } else {
        s1 <= s2
    }
}

fn sweep_for_center(
    sx: f64,
    sy: f64,
    ex: f64,
    ey: f64,
    cx: f64,
    cy: f64,
    clockwise: bool,
) -> f64 {
    let start_angle = (sy - cy).atan2(sx - cx);
    let end_angle = (ey - cy).atan2(ex - cx);
    let mut sweep = end_angle - start_angle;
    if clockwise {
        if sweep >= 0.0 {
            sweep -= std::f64::consts::TAU;
        }
    } else if sweep <= 0.0 {
        sweep += std::f64::consts::TAU;
    }
    sweep
}

fn arc_to_segments(
    start: Vec3,
    end: Vec3,
    center: Vec3,
    clockwise: bool,
    plane: Plane,
) -> Vec<LineSegment> {
    let (sx, sy) = plane_coords(start, plane);
    let (ex, ey) = plane_coords(end, plane);
    let (cx, cy) = plane_coords(center, plane);

    let radius = ((sx - cx).powi(2) + (sy - cy).powi(2)).sqrt();
    if radius.abs() < 1e-6 {
        return Vec::new();
    }

    let start_angle = (sy - cy).atan2(sx - cx);
    let end_angle = (ey - cy).atan2(ex - cx);

    let mut sweep = end_angle - start_angle;
    if clockwise {
        if sweep >= 0.0 {
            sweep -= std::f64::consts::TAU;
        }
    } else if sweep <= 0.0 {
        sweep += std::f64::consts::TAU;
    }

    let arc_length = radius * sweep.abs();
    let steps = (arc_length / ARC_SEGMENT_LENGTH).ceil() as usize;
    let steps = steps.max(1);

    let mut segments = Vec::with_capacity(steps);
    let mut prev = start;
    for step in 1..=steps {
        let t = step as f64 / steps as f64;
        let angle = start_angle + sweep * t;
        let px = cx + radius * angle.cos();
        let py = cy + radius * angle.sin();

        let mut point = start;
        match plane {
            Plane::XY => {
                point.x = px;
                point.y = py;
                point.z = start.z + (end.z - start.z) * t;
            }
            Plane::XZ => {
                point.x = px;
                point.z = py;
                point.y = start.y + (end.y - start.y) * t;
            }
            Plane::YZ => {
                point.y = px;
                point.z = py;
                point.x = start.x + (end.x - start.x) * t;
            }
        }

        segments.push(LineSegment {
            start: prev,
            end: point,
            kind: MoveKind::Feed,
        });
        prev = point;
    }

    segments
}

fn plane_coords(p: Vec3, plane: Plane) -> (f64, f64) {
    match plane {
        Plane::XY => (p.x, p.y),
        Plane::XZ => (p.x, p.z),
        Plane::YZ => (p.y, p.z),
    }
}

#[derive(Debug, Clone, Copy)]
struct Word {
    letter: char,
    value: f64,
}

fn parse_words(line: &str, options: &ParseOptions) -> Result<Vec<Word>> {
    let mut words = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.peek() {
        if ch.is_ascii_whitespace() {
            chars.next();
            continue;
        }
        if ch.is_ascii_alphabetic() {
            let letter = ch.to_ascii_uppercase();
            chars.next();
            let mut num = String::new();
            while let Some(next) = chars.peek() {
                if next.is_ascii_whitespace() || next.is_ascii_alphabetic() {
                    break;
                }
                num.push(*next);
                chars.next();
            }
            if num.trim().is_empty() {
                if options.should_ignore_missing(letter)
                    || (options.ignore_unknown_words && !is_known_letter(letter))
                {
                    continue;
                }
                return Err(anyhow!("missing value for {}", letter));
            }
            let value = num.trim().parse::<f64>()?;
            words.push(Word { letter, value });
        } else {
            chars.next();
        }
    }
    Ok(words)
}

fn strip_comments(line: &str) -> String {
    let mut out = String::new();
    let mut in_paren = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if in_paren {
            if ch == ')' {
                in_paren = false;
            }
            continue;
        }
        if ch == '(' {
            in_paren = true;
            continue;
        }
        if ch == ';' {
            break;
        }
        out.push(ch);
    }
    out
}

fn is_known_letter(letter: char) -> bool {
    matches!(letter, 'G' | 'X' | 'Y' | 'Z' | 'I' | 'J' | 'K' | 'R')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_linear_move() {
        let mut parser = Parser::new(ParseOptions::default());
        parser.parse_line("G1 X10 Y5", 1).unwrap();
        let toolpath = parser.finish().unwrap();
        assert_eq!(toolpath.segments.len(), 1);
        let seg = toolpath.segments[0];
        assert_eq!(seg.start, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(seg.end, Vec3::new(10.0, 5.0, 0.0));
    }

    #[test]
    fn parse_arc_move() {
        let mut parser = Parser::new(ParseOptions::default());
        parser.parse_line("G2 X10 Y0 I5 J0", 1).unwrap();
        let toolpath = parser.finish().unwrap();
        assert!(!toolpath.segments.is_empty());
    }

    #[test]
    fn parse_arc_move_radius() {
        let mut parser = Parser::new(ParseOptions::default());
        parser.parse_line("G2 X10 Y0 R5", 1).unwrap();
        let toolpath = parser.finish().unwrap();
        assert!(!toolpath.segments.is_empty());
    }

    #[test]
    fn ignore_missing_value_word() {
        let mut parser = Parser::new(ParseOptions::with_ignore_missing(['E']));
        parser.parse_line("T1 E", 1).unwrap();
        let toolpath = parser.finish().unwrap();
        assert_eq!(toolpath.segments.len(), 0);
    }

    #[test]
    fn ignore_unknown_missing_value() {
        let mut parser = Parser::new(ParseOptions::default().with_ignore_unknown_words(true));
        parser.parse_line("T1 E", 1).unwrap();
        let toolpath = parser.finish().unwrap();
        assert_eq!(toolpath.segments.len(), 0);
    }
}
