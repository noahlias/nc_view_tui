use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::text::{Line as TextLine, Span};
use ratatui::widgets::canvas::{Canvas, Line};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::{App, PanelFocus};
use cnc_geom::{project_point, ProjectionMode, ProjectionParams, Vec3, ViewAngles};
use cnc_gcode::MoveKind;

pub fn draw(frame: &mut Frame<'_>, app: &mut App) {
    let size = frame.size();
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(size);
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(main[0]);

    let metrics = app.compute_view_metrics(body[0]);
    app.last_metrics = Some(metrics);

    let theme = app.config.theme.clone();
    let canvas = Canvas::default()
        .marker(ratatui::symbols::Marker::HalfBlock)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Toolpath")
                .style(Style::default().bg(theme.background).fg(theme.foreground)),
        )
        .x_bounds([
            metrics.center.x - metrics.half_w,
            metrics.center.x + metrics.half_w,
        ])
        .y_bounds([
            metrics.center.y - metrics.half_h,
            metrics.center.y + metrics.half_h,
        ])
        .paint(|ctx| {
            let params = ProjectionParams {
                mode: app.view.projection,
                angles: ViewAngles {
                    yaw: app.view.yaw,
                    pitch: app.view.pitch,
                },
                camera_distance: metrics.camera_distance,
                target: metrics.target,
            };
            draw_plane(ctx, app, params);
            draw_grid(ctx, app, params);
            draw_axes(ctx, app, params);
            draw_toolpath(ctx, app, params);
        });

    frame.render_widget(canvas, body[0]);

    draw_hud_origin(frame, app, body[0]);
    draw_file_panel(frame, app, body[1]);

    let status = build_status_line(app);
    let status_widget = Paragraph::new(status).style(
        Style::default()
            .fg(theme.status_fg)
            .bg(theme.status_bg),
    );
    frame.render_widget(status_widget, main[1]);

    if app.show_help {
        draw_help_popup(frame, app, size);
    }
}

fn draw_toolpath(ctx: &mut ratatui::widgets::canvas::Context, app: &App, params: ProjectionParams) {
    let (start_idx, end_idx) = app.visible_segment_range();
    let total_visible = end_idx.saturating_sub(start_idx);
    let background = app.config.theme.background;
    for (idx, seg) in app
        .toolpath
        .segments
        .iter()
        .skip(start_idx)
        .take(total_visible)
        .enumerate()
    {
        let start = project_point(seg.start, params);
        let end = project_point(seg.end, params);
        let fade = segment_fade(idx, total_visible);
        let color = match seg.kind {
            MoveKind::Rapid => fade_color(app.config.theme.path_rapid, background, fade * 0.7),
            MoveKind::Feed => fade_color(app.config.theme.path_feed, background, fade),
        };
        ctx.draw(&Line {
            x1: start.x,
            y1: start.y,
            x2: end.x,
            y2: end.y,
            color,
        });
    }
}

fn draw_axes(ctx: &mut ratatui::widgets::canvas::Context, app: &App, params: ProjectionParams) {
    let bounds = app.toolpath.bounds;
    let size = bounds.size();
    let max_dim = size.x.max(size.y).max(size.z).max(1.0);
    let axis_len = max_dim * 0.4;

    let origin = Vec3::new(0.0, 0.0, 0.0);
    let x_end = Vec3::new(axis_len, 0.0, 0.0);
    let y_end = Vec3::new(0.0, axis_len, 0.0);
    let z_end = Vec3::new(0.0, 0.0, axis_len);

    let o = project_point(origin, params);
    let x = project_point(x_end, params);
    let y = project_point(y_end, params);
    let z = project_point(z_end, params);

    ctx.draw(&Line {
        x1: o.x,
        y1: o.y,
        x2: x.x,
        y2: x.y,
        color: app.config.theme.axis_x,
    });
    draw_arrow(ctx, o, x, app.config.theme.axis_x);
    ctx.draw(&Line {
        x1: o.x,
        y1: o.y,
        x2: y.x,
        y2: y.y,
        color: app.config.theme.axis_y,
    });
    draw_arrow(ctx, o, y, app.config.theme.axis_y);
    ctx.draw(&Line {
        x1: o.x,
        y1: o.y,
        x2: z.x,
        y2: z.y,
        color: app.config.theme.axis_z,
    });
    draw_arrow(ctx, o, z, app.config.theme.axis_z);
    ctx.print(
        x.x,
        x.y,
        TextLine::from(Span::styled("X", Style::default().fg(app.config.theme.axis_x))),
    );
    ctx.print(
        y.x,
        y.y,
        TextLine::from(Span::styled("Y", Style::default().fg(app.config.theme.axis_y))),
    );
    ctx.print(
        z.x,
        z.y,
        TextLine::from(Span::styled("Z", Style::default().fg(app.config.theme.axis_z))),
    );
}

fn draw_grid(ctx: &mut ratatui::widgets::canvas::Context, app: &App, params: ProjectionParams) {
    let bounds = app.toolpath.bounds;
    if !bounds.initialized {
        return;
    }

    let size = bounds.size();
    let max_dim = size.x.max(size.z).max(1.0);
    let step = (max_dim / 10.0).max(1.0);

    let start_x = (bounds.min.x / step).floor() * step;
    let end_x = (bounds.max.x / step).ceil() * step;
    let start_z = (bounds.min.z / step).floor() * step;
    let end_z = (bounds.max.z / step).ceil() * step;

    let mut x = start_x;
    while x <= end_x {
        let p1 = project_point(Vec3::new(x, 0.0, start_z), params);
        let p2 = project_point(Vec3::new(x, 0.0, end_z), params);
        ctx.draw(&Line {
            x1: p1.x,
            y1: p1.y,
            x2: p2.x,
            y2: p2.y,
            color: app.config.theme.grid,
        });
        x += step;
    }

    let mut z = start_z;
    while z <= end_z {
        let p1 = project_point(Vec3::new(start_x, 0.0, z), params);
        let p2 = project_point(Vec3::new(end_x, 0.0, z), params);
        ctx.draw(&Line {
            x1: p1.x,
            y1: p1.y,
            x2: p2.x,
            y2: p2.y,
            color: app.config.theme.grid,
        });
        z += step;
    }
}

fn draw_plane(ctx: &mut ratatui::widgets::canvas::Context, app: &App, params: ProjectionParams) {
    let bounds = app.toolpath.bounds;
    if !bounds.initialized {
        return;
    }
    let size = bounds.size();
    let max_dim = size.x.max(size.y).max(1.0);
    let step = (max_dim / 60.0).max(0.5);
    let z = bounds.min.z;
    let p1 = project_point(Vec3::new(bounds.min.x, bounds.min.y, z), params);
    let p2 = project_point(Vec3::new(bounds.max.x, bounds.min.y, z), params);
    let p3 = project_point(Vec3::new(bounds.max.x, bounds.max.y, z), params);
    let p4 = project_point(Vec3::new(bounds.min.x, bounds.max.y, z), params);
    let fill = fade_color(app.config.theme.grid, app.config.theme.background, 0.25);
    fill_polygon(ctx, &[p1, p2, p3, p4], fill, step);
}

fn build_status_line(app: &App) -> String {
    let file = app
        .file_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("<stdin>");
    let projection = match app.view.projection {
        ProjectionMode::Orthographic => "ortho",
        ProjectionMode::Perspective => "persp",
    };
    let focus = match app.file_panel.focus {
        PanelFocus::Viewport => "view",
        PanelFocus::File => "file",
    };
    let playback = if app.playback.active {
        if app.playback.playing {
            "play"
        } else {
            "pause"
        }
    } else {
        "off"
    };
    let visible = app.visible_segment_count();
    let (line_start, line_end) = app.file_panel.selection_range(app.file_lines.len());
    let mode = if app.file_panel.visual { "visual" } else { "single" };
    format!(
        "{} | sel:{}-{} | {} | seg:{}/{} | zoom:{:.2} | {} | {} | {}",
        file,
        line_start + 1,
        line_end + 1,
        mode,
        visible,
        app.toolpath.segments.len(),
        app.view.zoom,
        projection,
        playback,
        focus
    )
}

fn segment_fade(index: usize, total: usize) -> f64 {
    if total <= 1 {
        return 1.0;
    }
    let t = index as f64 / (total - 1) as f64;
    t.powf(0.6)
}

fn fade_color(base: ratatui::style::Color, background: ratatui::style::Color, t: f64) -> ratatui::style::Color {
    let t = t.clamp(0.0, 1.0);
    let base_rgb = color_to_rgb(base);
    let bg_rgb = color_to_rgb(background);
    if let (Some((br, bg, bb)), Some((rr, rg, rb))) = (base_rgb, bg_rgb) {
        let r = rr as f64 + (br as f64 - rr as f64) * t;
        let g = rg as f64 + (bg as f64 - rg as f64) * t;
        let b = rb as f64 + (bb as f64 - rb as f64) * t;
        return ratatui::style::Color::Rgb(r.round() as u8, g.round() as u8, b.round() as u8);
    }
    base
}

fn color_to_rgb(color: ratatui::style::Color) -> Option<(u8, u8, u8)> {
    match color {
        ratatui::style::Color::Black => Some((0, 0, 0)),
        ratatui::style::Color::Red => Some((205, 49, 49)),
        ratatui::style::Color::Green => Some((13, 188, 121)),
        ratatui::style::Color::Yellow => Some((229, 229, 16)),
        ratatui::style::Color::Blue => Some((36, 114, 200)),
        ratatui::style::Color::Magenta => Some((188, 63, 188)),
        ratatui::style::Color::Cyan => Some((17, 168, 205)),
        ratatui::style::Color::Gray => Some((153, 153, 153)),
        ratatui::style::Color::DarkGray => Some((102, 102, 102)),
        ratatui::style::Color::LightRed => Some((241, 76, 76)),
        ratatui::style::Color::LightGreen => Some((35, 209, 139)),
        ratatui::style::Color::LightYellow => Some((245, 245, 67)),
        ratatui::style::Color::LightBlue => Some((59, 142, 234)),
        ratatui::style::Color::LightMagenta => Some((214, 112, 214)),
        ratatui::style::Color::LightCyan => Some((41, 184, 219)),
        ratatui::style::Color::White => Some((229, 229, 229)),
        ratatui::style::Color::Rgb(r, g, b) => Some((r, g, b)),
        _ => None,
    }
}

fn apply_line_style(mut spans: Vec<Span<'_>>, style: Style) -> Vec<Span<'_>> {
    if style == Style::default() {
        return spans;
    }
    for span in spans.iter_mut() {
        span.style = span.style.patch(style);
    }
    spans
}

fn highlight_gcode_line(line: &str, theme: &crate::config::Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.peek() {
        if *ch == ';' {
            let rest: String = chars.by_ref().collect();
            spans.push(Span::styled(
                rest,
                Style::default().fg(theme.code_comment),
            ));
            break;
        }
        if *ch == '(' {
            let mut comment = String::new();
            while let Some(c) = chars.next() {
                comment.push(c);
                if c == ')' {
                    break;
                }
            }
            spans.push(Span::styled(
                comment,
                Style::default().fg(theme.code_comment),
            ));
            continue;
        }
        if ch.is_ascii_whitespace() {
            spans.push(Span::raw(ch.to_string()));
            chars.next();
            continue;
        }
        if ch.is_ascii_alphabetic() {
            let letter = ch.to_ascii_uppercase();
            chars.next();
            let mut number = String::new();
            while let Some(n) = chars.peek() {
                if n.is_ascii_digit() || *n == '.' || *n == '-' || *n == '+' {
                    number.push(*n);
                    chars.next();
                } else {
                    break;
                }
            }
            let letter_style = match letter {
                'G' | 'M' | 'F' | 'S' | 'T' => Style::default().fg(theme.code_keyword),
                'X' => Style::default().fg(theme.axis_x),
                'Y' => Style::default().fg(theme.axis_y),
                'Z' => Style::default().fg(theme.axis_z),
                'I' | 'J' | 'K' | 'R' => Style::default().fg(theme.code_axis),
                'N' | 'O' => Style::default().fg(theme.code_label),
                _ => Style::default().fg(theme.code_keyword),
            };
            spans.push(Span::styled(letter.to_string(), letter_style));
            if !number.is_empty() {
                spans.push(Span::styled(
                    number,
                    Style::default().fg(theme.code_number),
                ));
            }
            continue;
        }
        spans.push(Span::raw(ch.to_string()));
        chars.next();
    }
    spans
}

fn draw_file_panel(frame: &mut Frame<'_>, app: &mut App, area: ratatui::layout::Rect) {
    let theme = &app.config.theme;
    let focus_style = match app.file_panel.focus {
        PanelFocus::File => Style::default().fg(theme.axis_z),
        PanelFocus::Viewport => Style::default().fg(theme.grid),
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title("NC File")
        .style(focus_style);

    let inner = block.inner(area);
    let view_height = inner.height as usize;
    app.file_panel.view_height = view_height;
    app.file_panel.ensure_visible();

    let total = app.file_lines.len();
    let width = total.to_string().len().max(2);
    let show_numbers = app.config.ui.show_line_numbers;
    let start = app.file_panel.scroll.min(total);
    let end = (start + view_height).min(total);
    let (sel_start, sel_end) = app.file_panel.selection_range(total);
    let mut lines = Vec::new();
    for idx in start..end {
        let mut spans = Vec::new();
        if show_numbers {
            let number = format!("{:>width$} ", idx + 1, width = width);
            spans.push(Span::styled(number, Style::default().fg(theme.code_label)));
        }
        spans.extend(highlight_gcode_line(&app.file_lines[idx], theme));

        let mut line_style = Style::default();
        if idx >= sel_start && idx <= sel_end {
            line_style = line_style.bg(theme.grid);
        }
        if idx == app.file_panel.selected {
            line_style = line_style.bg(theme.axis_z).fg(theme.background);
        }
        let spans = apply_line_style(spans, line_style);
        lines.push(TextLine::from(spans));
    }

    let paragraph = Paragraph::new(lines).style(Style::default().bg(theme.background));
    frame.render_widget(block, area);
    frame.render_widget(paragraph, inner);
}

fn draw_arrow(
    ctx: &mut ratatui::widgets::canvas::Context,
    from: cnc_geom::Vec2,
    to: cnc_geom::Vec2,
    color: ratatui::style::Color,
) {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 1e-6 {
        return;
    }
    let arrow_len = (len * 0.08).clamp(1.0, 6.0);
    let angle = dy.atan2(dx);
    let phi = 25.0_f64.to_radians();
    let left = (
        to.x - arrow_len * (angle + phi).cos(),
        to.y - arrow_len * (angle + phi).sin(),
    );
    let right = (
        to.x - arrow_len * (angle - phi).cos(),
        to.y - arrow_len * (angle - phi).sin(),
    );
    ctx.draw(&Line {
        x1: to.x,
        y1: to.y,
        x2: left.0,
        y2: left.1,
        color,
    });
    ctx.draw(&Line {
        x1: to.x,
        y1: to.y,
        x2: right.0,
        y2: right.1,
        color,
    });
}

fn fill_polygon(
    ctx: &mut ratatui::widgets::canvas::Context,
    points: &[cnc_geom::Vec2],
    color: ratatui::style::Color,
    step: f64,
) {
    if points.len() < 3 {
        return;
    }
    let mut min_y = points[0].y;
    let mut max_y = points[0].y;
    for p in points.iter().skip(1) {
        min_y = min_y.min(p.y);
        max_y = max_y.max(p.y);
    }
    let mut y = min_y;
    let step = step.max(0.1);
    while y <= max_y {
        let mut xs = Vec::new();
        for i in 0..points.len() {
            let a = points[i];
            let b = points[(i + 1) % points.len()];
            let (y1, y2) = (a.y, b.y);
            if (y1 <= y && y < y2) || (y2 <= y && y < y1) {
                let t = (y - y1) / (y2 - y1);
                let x = a.x + t * (b.x - a.x);
                xs.push(x);
            }
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        for pair in xs.chunks(2) {
            if pair.len() == 2 {
                ctx.draw(&Line {
                    x1: pair[0],
                    y1: y,
                    x2: pair[1],
                    y2: y,
                    color,
                });
            }
        }
        y += step;
    }
}

fn draw_hud_origin(frame: &mut Frame<'_>, app: &App, area: ratatui::layout::Rect) {
    let theme = &app.config.theme;
    let width = 12;
    let height = 5;
    if area.width < width + 2 || area.height < height + 2 {
        return;
    }
    let rect = ratatui::layout::Rect {
        x: area.x + 1,
        y: area.y + 1,
        width,
        height,
    };
    frame.render_widget(Clear, rect);
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Origin")
        .style(Style::default().bg(theme.background).fg(theme.foreground));
    let inner = block.inner(rect);
    let lines = vec![
        TextLine::from(vec![
            Span::styled("X", Style::default().fg(theme.axis_x)),
            Span::raw("  →"),
        ]),
        TextLine::from(vec![
            Span::styled("Y", Style::default().fg(theme.axis_y)),
            Span::raw("  ↑"),
        ]),
        TextLine::from(vec![
            Span::styled("Z", Style::default().fg(theme.axis_z)),
            Span::raw("  •"),
        ]),
    ];
    frame.render_widget(block, rect);
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(theme.background)),
        inner,
    );
}

fn draw_help_popup(frame: &mut Frame<'_>, app: &App, area: ratatui::layout::Rect) {
    let theme = &app.config.theme;
    let width = area.width.saturating_sub(10).min(60).max(30);
    let height = area.height.saturating_sub(6).min(18).max(10);
    let rect = ratatui::layout::Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    };

    frame.render_widget(Clear, rect);
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Help")
        .style(Style::default().bg(theme.background).fg(theme.foreground));
    let inner = block.inner(rect);

    let key_style = Style::default().fg(theme.background).bg(theme.axis_z);
    let desc_style = Style::default().fg(theme.foreground);
    let mut lines = Vec::new();
    lines.push(TextLine::from(vec![
        Span::styled("Key", Style::default().fg(theme.axis_x)),
        Span::raw("  "),
        Span::styled("Description", Style::default().fg(theme.axis_x)),
    ]));
    lines.push(TextLine::from(""));
    let entries = [
        ("h/j/k/l", "Pan view"),
        ("w/s/a/d", "Rotate view"),
        ("+ / -", "Zoom in/out"),
        ("r", "Reset pan+zoom"),
        ("g", "Fit to toolpath"),
        ("p", "Toggle projection"),
        ("space", "Play/Pause animation"),
        ("tab", "Toggle focus (view/file)"),
        ("v", "Visual select (range)"),
        ("↑ / ↓", "Select file line"),
        ("PgUp/PgDn", "Page scroll"),
        ("q", "Quit"),
        ("?", "Close help"),
    ];
    let key_width = 10usize;
    for (key, desc) in entries {
        lines.push(help_line(key, desc, key_width, key_style, desc_style));
    }

    frame.render_widget(block, rect);
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(theme.background)),
        inner,
    );
}

fn help_line(
    key: &str,
    desc: &str,
    key_width: usize,
    key_style: Style,
    desc_style: Style,
) -> TextLine<'static> {
    let mut padded = key.to_string();
    if padded.len() < key_width {
        padded.push_str(&" ".repeat(key_width - padded.len()));
    }
    TextLine::from(vec![
        Span::styled(padded, key_style),
        Span::raw("  "),
        Span::styled(desc.to_string(), desc_style),
    ])
}
