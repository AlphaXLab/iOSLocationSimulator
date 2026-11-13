use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontFamily, FontId, Margin, Shadow, Stroke,
};

pub const SPACE_1: f32 = 4.0;
pub const SPACE_2: f32 = 8.0;
pub const SPACE_3: f32 = 12.0;
pub const SPACE_4: f32 = 16.0;
pub const SPACE_5: f32 = 24.0;

pub const RADIUS_SM: u8 = 0;
pub const RADIUS_MD: u8 = 0;
pub const RADIUS_LG: u8 = 0;

pub const ROW_H: f32 = 24.0;
pub const HEADER_H: f32 = 24.0;

pub const TOOLBAR_H: f32 = 44.0;
pub const TOOLBAR_H_MACOS: f32 = 52.0;
pub const TOOLBAR_ICON: f32 = 28.0;
pub const TOOLBAR_GLYPH: f32 = 17.0;
pub const TOOLBAR_ICON_GAP: f32 = 8.0;
pub const TOOLBAR_GROUP_EXTRA: f32 = 10.0;
pub const TOOLBAR_PILL_GAP: f32 = 20.0;
pub const TOOLBAR_PILL_H: f32 = 26.0;
pub const TOOLBAR_EDGE_PAD: f32 = 11.0;
pub const TOOLBAR_EDGE_PAD_RIGHT: f32 = 13.0;
pub const TOOLBAR_TRAFFIC_LIGHTS: f32 = 78.0;

pub fn toolbar_height() -> f32 {
    if cfg!(target_os = "macos") {
        TOOLBAR_H_MACOS
    } else {
        TOOLBAR_H
    }
}

pub fn toolbar_vert_pad() -> f32 {
    (toolbar_height() - TOOLBAR_ICON) / 2.0
}

pub const fn hex(val: u32) -> Color32 {
    Color32::from_rgb(
        ((val >> 16) & 0xff) as u8,
        ((val >> 8) & 0xff) as u8,
        (val & 0xff) as u8,
    )
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub canvas: Color32,
    pub sidebar: Color32,
    pub chrome: Color32,
    pub elevated: Color32,
    pub button: Color32,
    pub input: Color32,
    pub hover: Color32,
    pub selection: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub text_faint: Color32,
    pub separator: Color32,
    pub accent: Color32,
    pub danger: Color32,
    pub danger_bg: Color32,
    pub ok: Color32,
    pub ok_bg: Color32,
    pub header: Color32,
    pub zebra: Color32,
    pub grid_line: Color32,
}

pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
            canvas: hex(0x1e1e1e),
            sidebar: hex(0x252525),
            chrome: hex(0x202228),
            elevated: hex(0x2c2f36),
            button: hex(0x2a2d34),
            input: hex(0x16171b),
            hover: hex(0x333333),
            selection: hex(0x3a6284),
            text: hex(0xdedede),
            text_muted: hex(0x9a9a9a),
            text_faint: hex(0x6e6e6e),
            separator: hex(0x3a3a3a),
            accent: hex(0x4c8dff),
            danger: hex(0xff8b8b),
            danger_bg: hex(0x481c1c),
            ok: hex(0x3dd68c),
            ok_bg: hex(0x1c3a2c),
            header: hex(0x1d1d1d),
            zebra: hex(0x292929),
            grid_line: hex(0x3c3c3c),
        }
    } else {
        Palette {
            canvas: hex(0xffffff),
            sidebar: hex(0xf3f4f6),
            chrome: hex(0xf7f8fa),
            elevated: hex(0xffffff),
            button: hex(0xe8eaee),
            input: hex(0xf4f5f7),
            hover: hex(0xe4e7ee),
            selection: hex(0xd7e4f8),
            text: hex(0x1c1d21),
            text_muted: hex(0x5c6573),
            text_faint: hex(0x8b93a1),
            separator: hex(0xe1e3e8),
            accent: hex(0x1d5fd1),
            danger: hex(0xb42318),
            danger_bg: hex(0xfde8e8),
            ok: hex(0x0e7a43),
            ok_bg: hex(0xe3f6eb),
            header: hex(0xe4e7ee),
            zebra: hex(0xf3f5f8),
            grid_line: hex(0xc5cad4),
        }
    }
}

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Inter".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "InterSemi".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-SemiBold.ttf"
        ))),
    );
    fonts.font_data.insert(
        "JetBrainsMono".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/JetBrainsMono-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "NotoSansMyanmar".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/NotoSansMyanmar.ttc"
        ))),
    );

    // Dynamic system fallback fonts for CJK, Arabic, Thai, Devanagari, and full Unicode
    let fallback_paths = [
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/AppleSDGothicNeo.ttc",
        "/System/Library/Fonts/Supplemental/Thonburi.ttc",
        "/System/Library/Fonts/NotoNastaliq.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "C:\\Windows\\Fonts\\arialuni.ttf",
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\seguiemj.ttf",
    ];

    let mut sys_fallbacks = vec!["NotoSansMyanmar".to_owned()];
    for (idx, path) in fallback_paths.iter().enumerate() {
        if let Ok(bytes) = std::fs::read(path) {
            let name = format!("SysUnicode_{idx}");
            fonts
                .font_data
                .insert(name.clone(), Arc::new(FontData::from_owned(bytes)));
            sys_fallbacks.push(name);
        }
    }

    // Proportional font family
    let prop = fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap();
    prop.insert(0, "Inter".to_owned());
    prop.extend(sys_fallbacks.iter().cloned());

    // Monospace font family
    let mono = fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap();
    mono.insert(0, "JetBrainsMono".to_owned());
    mono.extend(sys_fallbacks.iter().cloned());

    // InterSemi family (with full fallbacks)
    let mut semi_chain = vec![
        "InterSemi".to_owned(),
        "Inter".to_owned(),
        "Ubuntu-Light".to_owned(),
        "NotoEmoji-Regular".to_owned(),
    ];
    semi_chain.extend(sys_fallbacks.iter().cloned());
    fonts.families.insert(
        FontFamily::Name("InterSemi".into()),
        semi_chain,
    );

    ctx.set_fonts(fonts);
}

pub fn semi(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("InterSemi".into()))
}

pub fn ui(size: f32) -> FontId {
    FontId::proportional(size)
}

pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

pub fn apply(ctx: &egui::Context, dark: bool) {
    ctx.set_visuals(visuals(dark));
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(SPACE_2, SPACE_1);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        style.spacing.interact_size = egui::vec2(28.0, 20.0);
        style.spacing.menu_margin = Margin::same(6);
        style.spacing.window_margin = Margin::same(12);
        style.spacing.indent = 14.0;
        style.text_styles.insert(egui::TextStyle::Small, ui(11.0));
        style.text_styles.insert(egui::TextStyle::Body, ui(13.0));
        style.text_styles.insert(egui::TextStyle::Button, ui(13.0));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, semi(18.0));
        style
            .text_styles
            .insert(egui::TextStyle::Monospace, mono(13.0));
    });
}

fn visuals(dark: bool) -> egui::Visuals {
    let pal = palette(dark);
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.window_fill = pal.chrome;
    visuals.panel_fill = pal.sidebar;
    visuals.extreme_bg_color = pal.input;
    visuals.faint_bg_color = pal.elevated;
    visuals.code_bg_color = pal.input;
    visuals.selection.bg_fill = pal.selection;
    visuals.hyperlink_color = pal.accent;
    visuals.error_fg_color = pal.danger;
    visuals.warn_fg_color = hex(0xe6c07b);
    let text = Stroke::new(1.0_f32, pal.text);
    let hairline = Stroke::new(1.0_f32, pal.separator);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.fg_stroke = text;
        widget.bg_stroke = hairline;
        widget.corner_radius = CornerRadius::ZERO;
    }
    visuals.widgets.noninteractive.bg_fill = pal.sidebar;
    visuals.widgets.inactive.bg_fill = pal.button;
    visuals.widgets.hovered.bg_fill = pal.hover;
    visuals.widgets.active.bg_fill = pal.selection;
    visuals.widgets.open.bg_fill = pal.elevated;
    visuals.widgets.open.weak_bg_fill = pal.chrome;
    visuals.widgets.noninteractive.weak_bg_fill = pal.chrome;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, pal.text);
    visuals.menu_corner_radius = CornerRadius::ZERO;
    visuals.window_corner_radius = CornerRadius::ZERO;
    visuals.selection.bg_fill = pal.selection;
    visuals.selection.stroke = Stroke::new(1.0_f32, pal.accent);
    visuals.window_stroke = Stroke::new(1.0_f32, pal.separator);
    visuals.window_shadow = Shadow {
        offset: [0, 10],
        blur: 24,
        spread: 0,
        color: if dark {
            Color32::from_black_alpha(120)
        } else {
            Color32::from_black_alpha(40)
        },
    };
    visuals
}
