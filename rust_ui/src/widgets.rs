use crate::icons::{self, Icon};
use crate::theme::{self, Palette};
use eframe::egui::{
    self, Color32, CornerRadius, Rect, Sense, Stroke, StrokeKind,
};

pub fn colors(ui: &egui::Ui) -> Palette {
    theme::palette(ui.visuals().dark_mode)
}

pub fn icon_button(
    ui: &mut egui::Ui,
    icon: Icon,
    tip: &str,
    active: bool,
    size: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), Sense::click());
    let pal = colors(ui);
    let bg = if active {
        pal.selection
    } else if response.hovered() || response.highlighted() {
        pal.hover
    } else {
        Color32::TRANSPARENT
    };
    if bg != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), bg);
    }
    let color = if active { pal.accent } else { pal.text_muted };
    let glyph = if size >= 28.0 {
        16.0
    } else {
        (size * 0.66).clamp(11.0, 16.0)
    };
    let icon_rect = Rect::from_center_size(rect.center(), egui::vec2(glyph, glyph));
    let paint_color = if response.hovered() && !active {
        pal.text
    } else {
        color
    };
    icons::paint(ui.painter(), icon_rect, icon, paint_color);
    response.on_hover_text(tip)
}

pub fn toolbar_icon(ui: &mut egui::Ui, icon: Icon, tip: &str, active: bool) -> egui::Response {
    let size = theme::TOOLBAR_ICON;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), Sense::click());
    let pal = colors(ui);
    if response.hovered() || response.is_pointer_button_down_on() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), pal.hover);
    }
    let color = if active {
        pal.accent
    } else if response.hovered() {
        pal.text
    } else {
        pal.text_muted
    };
    let glyph = theme::TOOLBAR_GLYPH;
    let icon_rect = Rect::from_center_size(rect.center().round(), egui::vec2(glyph, glyph));
    icons::paint(ui.painter(), icon_rect, icon, color);
    response.on_hover_text(tip)
}

pub fn search_field(
    ui: &mut egui::Ui,
    text: &mut String,
    hint: &str,
    width: f32,
) -> egui::Response {
    let pal = colors(ui);
    let height = 28.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click());

    // Paint background well
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), pal.input);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0_f32, pal.separator),
        StrokeKind::Inside,
    );

    // Search Icon on left
    let icon_rect = Rect::from_min_size(rect.min + egui::vec2(8.0, 6.0), egui::vec2(16.0, 16.0));
    icons::paint(ui.painter(), icon_rect, Icon::Search, pal.text_muted);

    // TextEdit inside
    let edit_rect = Rect::from_min_max(
        rect.min + egui::vec2(28.0, 4.0),
        rect.max - egui::vec2(8.0, 4.0),
    );

    let edit = ui.put(
        edit_rect,
        egui::TextEdit::singleline(text)
            .hint_text(hint)
            .frame(false)
            .font(egui::TextStyle::Body),
    );

    if edit.has_focus() {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(theme::RADIUS_MD),
            Stroke::new(1.0_f32, pal.accent),
            StrokeKind::Inside,
        );
    }

    edit.union(response)
}
