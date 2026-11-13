//! Toolbar and sidebar icons.
//!
//! Geometry is Lucide (ISC, https://lucide.dev): 24×24 viewBox, stroke width 2,
//! drawn as vector paths at a fixed optical size so the stroke stays even.

use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Stroke, StrokeKind};

mod paths {
    include!("icon_paths.rs");
}

#[derive(Clone, Copy)]
pub enum Icon {
    Plus,
    Search,
    ChevronDown,
    ChevronRight,
    ChevronLeft,
    #[allow(dead_code)]
    ChevronsUpDown,
    Table,
    View,
    Function,
    SidebarLeft,
    SidebarRight,
    Trash,
    Play,
    Refresh,
    Backup,
    Restore,
    Plug,
    Sun,
    Moon,
    Close,
    Home,
    Query,
    Import,
    Sliders,
    CircleX,
    Eye,
    ListArrow,
    Lock,
    Unlock,
    Database,
    BarChart,
    More,
    PanelBottom,
    Split,
}

pub fn paint(painter: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    let stroke_w = (rect.width() * (2.0 / 24.0)).max(1.15);
    let stroke = Stroke::new(stroke_w, color);
    for mark in paths::marks(icon) {
        match *mark {
            paths::Mark::D(d) => {
                for sub in parse_path(d) {
                    stroke_subpath(painter, rect, &sub, stroke);
                }
            }
            paths::Mark::Circle(cx, cy, r) => {
                painter.circle_stroke(
                    map(rect, cx as f32, cy as f32),
                    r as f32 / 24.0 * rect.width(),
                    stroke,
                );
            }
            paths::Mark::Dot(cx, cy, r) => {
                painter.circle_filled(
                    map(rect, cx as f32, cy as f32),
                    r as f32 / 24.0 * rect.width(),
                    color,
                );
            }
            paths::Mark::Ellipse(cx, cy, rx, ry) => {
                painter.add(egui::epaint::EllipseShape::stroke(
                    map(rect, cx as f32, cy as f32),
                    egui::vec2(
                        rx as f32 / 24.0 * rect.width(),
                        ry as f32 / 24.0 * rect.height(),
                    ),
                    stroke,
                ));
            }
            paths::Mark::Rect(x, y, w, h, rx) => {
                let body = Rect::from_min_max(
                    map(rect, x as f32, y as f32),
                    map(rect, (x + w) as f32, (y + h) as f32),
                );
                let radius = CornerRadius::same((rx as f32 / 24.0 * rect.width()).round() as u8);
                painter.rect_stroke(body, radius, stroke, StrokeKind::Middle);
            }
        }
    }
}

fn map(rect: Rect, x: f32, y: f32) -> Pos2 {
    rect.min + egui::vec2(x / 24.0 * rect.width(), y / 24.0 * rect.height())
}

struct Subpath {
    pts: Vec<(f32, f32)>,
    closed: bool,
}

fn stroke_subpath(painter: &egui::Painter, rect: Rect, sub: &Subpath, stroke: Stroke) {
    if sub.pts.len() < 2 {
        return;
    }
    let points: Vec<Pos2> = sub
        .pts
        .iter()
        .copied()
        .map(|(x, y)| map(rect, x, y))
        .collect();
    let shape = if sub.closed {
        egui::epaint::PathShape::closed_line(points.clone(), stroke)
    } else {
        egui::epaint::PathShape::line(points.clone(), stroke)
    };
    painter.add(shape);
    let cap = stroke.width * 0.5;
    let color = stroke.color;
    if !sub.closed {
        painter.circle_filled(points[0], cap, color);
        painter.circle_filled(*points.last().unwrap(), cap, color);
    }
    for index in 1..points.len().saturating_sub(1) {
        let prev = points[index] - points[index - 1];
        let next = points[index + 1] - points[index];
        let len = prev.length() * next.length();
        if len > 0.0 && prev.dot(next) / len < 0.55 {
            painter.circle_filled(points[index], cap, color);
        }
    }
}

fn parse_path(d: &str) -> Vec<Subpath> {
    let bytes = d.as_bytes();
    let mut i = 0;
    let mut cmd = b' ';
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut sx = 0.0;
    let mut sy = 0.0;
    let mut current: Vec<(f32, f32)> = Vec::new();
    let mut out = Vec::new();
    let mut cubic_ctrl: Option<(f32, f32)> = None;
    let mut quad_ctrl: Option<(f32, f32)> = None;

    while i < bytes.len() {
        i = skip_sep(bytes, i);
        if i >= bytes.len() {
            break;
        }
        if bytes[i].is_ascii_alphabetic() {
            cmd = bytes[i];
            i += 1;
            if cmd == b'Z' || cmd == b'z' {
                if current.len() >= 2 {
                    let start = current[0];
                    if current.last().copied() != Some(start) {
                        current.push(start);
                    }
                    out.push(Subpath {
                        pts: std::mem::take(&mut current),
                        closed: true,
                    });
                } else {
                    current.clear();
                }
                cx = sx;
                cy = sy;
                cubic_ctrl = None;
                quad_ctrl = None;
                continue;
            }
        }
        if cmd == b' ' {
            break;
        }
        let rel = cmd.is_ascii_lowercase();
        match cmd {
            b'M' | b'm' => {
                let (mut x, ni) = parse_num(bytes, i);
                let (mut y, ni) = parse_num(bytes, ni);
                i = ni;
                if rel {
                    x += cx;
                    y += cy;
                }
                flush_open(&mut current, &mut out);
                current.push((x, y));
                cx = x;
                cy = y;
                sx = x;
                sy = y;
                cubic_ctrl = None;
                quad_ctrl = None;
                cmd = if rel { b'l' } else { b'L' };
            }
            b'L' | b'l' => {
                let (mut x, ni) = parse_num(bytes, i);
                let (mut y, ni) = parse_num(bytes, ni);
                i = ni;
                if rel {
                    x += cx;
                    y += cy;
                }
                push_pt(&mut current, (x, y));
                cx = x;
                cy = y;
                cubic_ctrl = None;
                quad_ctrl = None;
            }
            b'H' | b'h' => {
                let (mut x, ni) = parse_num(bytes, i);
                i = ni;
                if rel {
                    x += cx;
                }
                push_pt(&mut current, (x, cy));
                cx = x;
                cubic_ctrl = None;
                quad_ctrl = None;
            }
            b'V' | b'v' => {
                let (mut y, ni) = parse_num(bytes, i);
                i = ni;
                if rel {
                    y += cy;
                }
                push_pt(&mut current, (cx, y));
                cy = y;
                cubic_ctrl = None;
                quad_ctrl = None;
            }
            b'C' | b'c' => {
                let (mut x1, ni) = parse_num(bytes, i);
                let (mut y1, ni) = parse_num(bytes, ni);
                let (mut x2, ni) = parse_num(bytes, ni);
                let (mut y2, ni) = parse_num(bytes, ni);
                let (mut x, ni) = parse_num(bytes, ni);
                let (mut y, ni) = parse_num(bytes, ni);
                i = ni;
                if rel {
                    x1 += cx;
                    y1 += cy;
                    x2 += cx;
                    y2 += cy;
                    x += cx;
                    y += cy;
                }
                flatten_cubic(&mut current, (cx, cy), (x1, y1), (x2, y2), (x, y));
                cubic_ctrl = Some((x2, y2));
                quad_ctrl = None;
                cx = x;
                cy = y;
            }
            b'S' | b's' => {
                let (mut x2, ni) = parse_num(bytes, i);
                let (mut y2, ni) = parse_num(bytes, ni);
                let (mut x, ni) = parse_num(bytes, ni);
                let (mut y, ni) = parse_num(bytes, ni);
                i = ni;
                if rel {
                    x2 += cx;
                    y2 += cy;
                    x += cx;
                    y += cy;
                }
                let (x1, y1) = reflect(cubic_ctrl, cx, cy);
                flatten_cubic(&mut current, (cx, cy), (x1, y1), (x2, y2), (x, y));
                cubic_ctrl = Some((x2, y2));
                quad_ctrl = None;
                cx = x;
                cy = y;
            }
            b'Q' | b'q' => {
                let (mut qx, ni) = parse_num(bytes, i);
                let (mut qy, ni) = parse_num(bytes, ni);
                let (mut x, ni) = parse_num(bytes, ni);
                let (mut y, ni) = parse_num(bytes, ni);
                i = ni;
                if rel {
                    qx += cx;
                    qy += cy;
                    x += cx;
                    y += cy;
                }
                let (c1, c2) = quad_to_cubic((cx, cy), (qx, qy), (x, y));
                flatten_cubic(&mut current, (cx, cy), c1, c2, (x, y));
                quad_ctrl = Some((qx, qy));
                cubic_ctrl = None;
                cx = x;
                cy = y;
            }
            b'T' | b't' => {
                let (mut x, ni) = parse_num(bytes, i);
                let (mut y, ni) = parse_num(bytes, ni);
                i = ni;
                if rel {
                    x += cx;
                    y += cy;
                }
                let q = reflect(quad_ctrl, cx, cy);
                let (c1, c2) = quad_to_cubic((cx, cy), q, (x, y));
                flatten_cubic(&mut current, (cx, cy), c1, c2, (x, y));
                quad_ctrl = Some(q);
                cubic_ctrl = None;
                cx = x;
                cy = y;
            }
            b'A' | b'a' => {
                let (rx, ni) = parse_num(bytes, i);
                let (ry, ni) = parse_num(bytes, ni);
                let (angle, ni) = parse_num(bytes, ni);
                let (large, ni) = parse_flag(bytes, ni);
                let (sweep, ni) = parse_flag(bytes, ni);
                let (mut x, ni) = parse_num(bytes, ni);
                let (mut y, ni) = parse_num(bytes, ni);
                i = ni;
                if rel {
                    x += cx;
                    y += cy;
                }
                for cubic in arc_cubics((cx, cy), rx, ry, angle, large, sweep, (x, y)) {
                    flatten_cubic(&mut current, cubic.0, cubic.1, cubic.2, cubic.3);
                }
                cubic_ctrl = None;
                quad_ctrl = None;
                cx = x;
                cy = y;
            }
            _ => break,
        }
    }
    flush_open(&mut current, &mut out);
    out
}

fn flush_open(current: &mut Vec<(f32, f32)>, out: &mut Vec<Subpath>) {
    if current.len() >= 2 {
        out.push(Subpath {
            pts: std::mem::take(current),
            closed: false,
        });
    } else {
        current.clear();
    }
}

fn push_pt(pts: &mut Vec<(f32, f32)>, p: (f32, f32)) {
    if pts
        .last()
        .is_none_or(|prev| (prev.0 - p.0).abs() + (prev.1 - p.1).abs() > 0.04)
    {
        pts.push(p);
    }
}

fn reflect(ctrl: Option<(f32, f32)>, cx: f32, cy: f32) -> (f32, f32) {
    match ctrl {
        Some((x, y)) => (2.0 * cx - x, 2.0 * cy - y),
        None => (cx, cy),
    }
}

fn quad_to_cubic(p0: (f32, f32), q: (f32, f32), p: (f32, f32)) -> ((f32, f32), (f32, f32)) {
    let c1 = (
        p0.0 + (2.0 / 3.0) * (q.0 - p0.0),
        p0.1 + (2.0 / 3.0) * (q.1 - p0.1),
    );
    let c2 = (
        p.0 + (2.0 / 3.0) * (q.0 - p.0),
        p.1 + (2.0 / 3.0) * (q.1 - p.1),
    );
    (c1, c2)
}

fn flatten_cubic(
    pts: &mut Vec<(f32, f32)>,
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
) {
    const N: i32 = 8;
    for step in 1..=N {
        let t = step as f32 / N as f32;
        let u = 1.0 - t;
        let x =
            u * u * u * p0.0 + 3.0 * u * u * t * p1.0 + 3.0 * u * t * t * p2.0 + t * t * t * p3.0;
        let y =
            u * u * u * p0.1 + 3.0 * u * u * t * p1.1 + 3.0 * u * t * t * p2.1 + t * t * t * p3.1;
        push_pt(pts, (x, y));
    }
}

fn skip_sep(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\n' | b'\r' | b',') {
        i += 1;
    }
    i
}

fn parse_num(bytes: &[u8], mut i: usize) -> (f32, usize) {
    i = skip_sep(bytes, i);
    if i >= bytes.len() {
        return (0.0, i);
    }
    let start = i;
    if matches!(bytes[i], b'+' | b'-') {
        i += 1;
    }
    let mut dotted = false;
    let mut exp = false;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_digit() {
            i += 1;
            continue;
        }
        if c == b'.' && !dotted && !exp {
            dotted = true;
            i += 1;
            continue;
        }
        if (c == b'e' || c == b'E') && !exp {
            exp = true;
            i += 1;
            if i < bytes.len() && matches!(bytes[i], b'+' | b'-') {
                i += 1;
            }
            continue;
        }
        break;
    }
    if i == start || (i == start + 1 && matches!(bytes[start], b'+' | b'-')) {
        return (0.0, start);
    }
    let text = std::str::from_utf8(&bytes[start..i]).unwrap_or("0");
    (text.parse().unwrap_or(0.0), i)
}

fn parse_flag(bytes: &[u8], mut i: usize) -> (bool, usize) {
    i = skip_sep(bytes, i);
    if i >= bytes.len() {
        return (false, i);
    }
    let flag = bytes[i] == b'1';
    if matches!(bytes[i], b'0' | b'1') {
        i += 1;
    }
    (flag, i)
}

fn arc_cubics(
    from: (f32, f32),
    mut rx: f32,
    mut ry: f32,
    angle_deg: f32,
    large: bool,
    sweep: bool,
    to: (f32, f32),
) -> Vec<((f32, f32), (f32, f32), (f32, f32), (f32, f32))> {
    let (x1, y1) = from;
    let (x2, y2) = to;
    if (x1 - x2).abs() < f32::EPSILON && (y1 - y2).abs() < f32::EPSILON {
        return Vec::new();
    }
    rx = rx.abs();
    ry = ry.abs();
    if rx < f32::EPSILON || ry < f32::EPSILON {
        return vec![(from, from, to, to)];
    }
    let phi = angle_deg.to_radians();
    let (cos_phi, sin_phi) = (phi.cos(), phi.sin());
    let dx = (x1 - x2) / 2.0;
    let dy = (y1 - y2) / 2.0;
    let x1p = cos_phi * dx + sin_phi * dy;
    let y1p = -sin_phi * dx + cos_phi * dy;
    let lam = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    if lam > 1.0 {
        let scale = lam.sqrt();
        rx *= scale;
        ry *= scale;
    }
    let sign = if large == sweep { -1.0 } else { 1.0 };
    let num = (rx * rx * ry * ry - rx * rx * y1p * y1p - ry * ry * x1p * x1p)
        / (rx * rx * y1p * y1p + ry * ry * x1p * x1p);
    let coef = sign * num.max(0.0).sqrt();
    let cxp = coef * rx * y1p / ry;
    let cyp = coef * -ry * x1p / rx;
    let cx = cos_phi * cxp - sin_phi * cyp + (x1 + x2) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (y1 + y2) / 2.0;
    let theta1 = vector_angle((1.0, 0.0), ((x1p - cxp) / rx, (y1p - cyp) / ry));
    let mut dtheta = vector_angle(
        ((x1p - cxp) / rx, (y1p - cyp) / ry),
        ((-x1p - cxp) / rx, (-y1p - cyp) / ry),
    );
    if !sweep && dtheta > 0.0 {
        dtheta -= std::f32::consts::TAU;
    }
    if sweep && dtheta < 0.0 {
        dtheta += std::f32::consts::TAU;
    }
    let chunks = (dtheta.abs() / std::f32::consts::FRAC_PI_2).ceil().max(1.0) as i32;
    let mut cubics = Vec::with_capacity(chunks as usize);
    for step in 0..chunks {
        let t0 = theta1 + dtheta * (step as f32 / chunks as f32);
        let t1 = theta1 + dtheta * ((step + 1) as f32 / chunks as f32);
        cubics.push(arc_chunk(cx, cy, rx, ry, phi, t0, t1));
    }
    cubics
}

fn vector_angle(u: (f32, f32), v: (f32, f32)) -> f32 {
    let sign = if u.0 * v.1 - u.1 * v.0 < 0.0 {
        -1.0
    } else {
        1.0
    };
    let dot = (u.0 * v.0 + u.1 * v.1) / (u.0.hypot(u.1) * v.0.hypot(v.1)).max(1e-6);
    sign * dot.clamp(-1.0, 1.0).acos()
}

fn arc_chunk(
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    phi: f32,
    t0: f32,
    t1: f32,
) -> ((f32, f32), (f32, f32), (f32, f32), (f32, f32)) {
    let delta = t1 - t0;
    let k = (4.0 / 3.0) * (delta / 4.0).tan();
    let p0 = (t0.cos(), t0.sin());
    let p1 = (p0.0 - k * t0.sin(), p0.1 + k * t0.cos());
    let p3 = (t1.cos(), t1.sin());
    let p2 = (p3.0 + k * t1.sin(), p3.1 - k * t1.cos());
    let map_pt = |p: (f32, f32)| {
        (
            cx + rx * phi.cos() * p.0 - ry * phi.sin() * p.1,
            cy + rx * phi.sin() * p.0 + ry * phi.cos() * p.1,
        )
    };
    (map_pt(p0), map_pt(p1), map_pt(p2), map_pt(p3))
}

#[cfg(test)]
mod tests {
    use super::parse_path;

    #[test]
    fn lines_and_relative_numbers_parse() {
        let path = parse_path("M4 7h8");
        assert_eq!(path.len(), 1);
        assert!((path[0].pts[0].0 - 4.0).abs() < 0.01);
        assert!((path[0].pts.last().unwrap().0 - 12.0).abs() < 0.01);

        let handle = parse_path("m21 21-4.34-4.34");
        assert_eq!(handle.len(), 1);
        let end = handle[0].pts.last().unwrap();
        assert!((end.0 - 16.66).abs() < 0.05);
        assert!((end.1 - 16.66).abs() < 0.05);
    }

    #[test]
    fn arcs_in_the_database_and_eye_stay_in_the_viewbox() {
        let db = parse_path("M3 5V19A9 3 0 0 0 21 19V5");
        assert!(!db.is_empty());
        for sub in &db {
            for (x, y) in &sub.pts {
                assert!((0.0..24.5).contains(x), "{x}");
                assert!((0.0..24.5).contains(y), "{y}");
            }
        }
        let eye = parse_path(
            "M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0",
        );
        assert!(eye[0].pts.len() > 8);
    }
}
