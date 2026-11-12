use crate::daemon::{DaemonClient, DaemonEvent};
use crate::geocoding::{parse_coordinates, GeocodingClient};
use crate::models::{GPXRoute, SavedLocation, SearchSuggestion, TunnelStatus, iOSDevice};
use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId,
    Layout, Margin, Pos2, Rect, Stroke, StrokeKind, Vec2,
};
use std::sync::Arc;
use walkers::{lat_lon, HttpOptions, HttpTiles, Map, MapMemory, Position, TileId};

#[derive(PartialEq, Clone, Copy)]
enum SidebarTab {
    Locations,
    Routes,
}

// ---------------------------------------------------------------------------
// CartoDB Dark Matter Tile Source (High-Performance Dark Map)
// ---------------------------------------------------------------------------
#[derive(Clone, Copy, Default)]
pub struct CartoDark;

impl walkers::sources::TileSource for CartoDark {
    fn tile_url(&self, tile_id: TileId) -> String {
        let servers = ['a', 'b', 'c', 'd'];
        let s = servers[(tile_id.x + tile_id.y) as usize % servers.len()];
        format!(
            "https://{}.basemaps.cartocdn.com/dark_all/{}/{}/{}.png",
            s, tile_id.zoom, tile_id.x, tile_id.y
        )
    }

    fn attribution(&self) -> walkers::sources::Attribution {
        walkers::sources::Attribution {
            text: "© OpenStreetMap contributors © CARTO",
            url: "https://carto.com/attributions",
            logo_light: None,
            logo_dark: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Professional macOS Dark Theme Tokens
// ---------------------------------------------------------------------------
#[allow(dead_code)]
struct DarkTheme;
#[allow(dead_code)]
impl DarkTheme {
    // Surfaces
    pub const WINDOW_BG: Color32 = Color32::from_rgb(14, 16, 21);
    pub const SIDEBAR_BG: Color32 = Color32::from_rgb(18, 21, 28);
    pub const HEADER_BG: Color32 = Color32::from_rgb(18, 21, 28);
    pub const BOTTOM_BG: Color32 = Color32::from_rgb(18, 21, 28);
    pub const CARD_BG: Color32 = Color32::from_rgb(26, 30, 41);
    pub const CARD_HOVER: Color32 = Color32::from_rgb(34, 40, 54);
    pub const CARD_SELECTED: Color32 = Color32::from_rgb(22, 38, 64);
    pub const INSET_BG: Color32 = Color32::from_rgb(11, 13, 18);

    // Borders
    pub const BORDER: Color32 = Color32::from_rgb(38, 44, 60);
    pub const BORDER_LIGHT: Color32 = Color32::from_rgb(52, 60, 80);
    pub const BORDER_ACTIVE: Color32 = Color32::from_rgb(0, 122, 255);

    // Typography
    pub const TEXT_TITLE: Color32 = Color32::from_rgb(248, 250, 252);
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(226, 232, 240);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139);
    pub const TEXT_CYAN: Color32 = Color32::from_rgb(56, 189, 248);

    // Accents
    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(0, 122, 255);
    pub const ACCENT_BLUE_HOVER: Color32 = Color32::from_rgb(30, 144, 255);
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(56, 189, 248);
    pub const ACCENT_GREEN: Color32 = Color32::from_rgb(34, 197, 94);
    pub const ACCENT_GREEN_BG: Color32 = Color32::from_rgb(20, 48, 30);
    pub const ACCENT_RED: Color32 = Color32::from_rgb(239, 68, 68);
    pub const ACCENT_RED_BG: Color32 = Color32::from_rgb(48, 20, 24);
    pub const ACCENT_AMBER: Color32 = Color32::from_rgb(245, 158, 11);
}

pub struct LocationSimulatorApp {
    daemon: DaemonClient,
    geocoding: GeocodingClient,

    // Map
    tiles: HttpTiles,
    map_memory: MapMemory,
    pos: Position,

    // App Data
    devices: Vec<iOSDevice>,
    selected_udid: Option<String>,
    saved_locations: Vec<SavedLocation>,
    gpx_routes: Vec<GPXRoute>,
    tunnel_status: TunnelStatus,

    // Navigation
    active_tab: SidebarTab,

    // Search
    search_query: String,
    last_searched_query: String,
    suggestions: Vec<SearchSuggestion>,
    show_suggestions: bool,

    // Status
    status_message: String,
    status_is_error: bool,

    // Route Simulation
    route_state: String,
    route_progress_current: usize,
    route_progress_total: usize,
    route_speed: f64,
    route_loop: bool,

    // Modals
    show_password_modal: bool,
    password_input: String,
    show_save_modal: bool,
    save_name_input: String,
    save_category_input: String,
    show_about_modal: bool,
}

impl LocationSimulatorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::apply_fonts_and_dark_theme(&cc.egui_ctx);

        let daemon = DaemonClient::spawn().expect("Failed to initialize Python daemon sidecar");
        let geocoding = GeocodingClient::new();

        let http_options = HttpOptions {
            cache: dirs_cache_dir(),
            ..Default::default()
        };
        // Use CartoDB Dark Matter tiles by default for dark theme
        let tiles = HttpTiles::with_options(
            CartoDark,
            http_options,
            cc.egui_ctx.to_owned(),
        );

        let initial_lat = 37.7749;
        let initial_lon = -122.4194;
        let pos = lat_lon(initial_lat, initial_lon);

        Self {
            daemon,
            geocoding,
            tiles,
            map_memory: MapMemory::default(),
            pos,
            devices: Vec::new(),
            selected_udid: None,
            saved_locations: Vec::new(),
            gpx_routes: Vec::new(),
            tunnel_status: TunnelStatus {
                status: None,
                running: false,
                error: None,
            },
            active_tab: SidebarTab::Locations,
            search_query: String::new(),
            last_searched_query: String::new(),
            suggestions: Vec::new(),
            show_suggestions: false,
            status_message: "Ready to simulate".to_string(),
            status_is_error: false,
            route_state: "idle".to_string(),
            route_progress_current: 0,
            route_progress_total: 0,
            route_speed: 1.0,
            route_loop: false,
            show_password_modal: false,
            password_input: String::new(),
            show_save_modal: false,
            save_name_input: String::new(),
            save_category_input: "other".to_string(),
            show_about_modal: false,
        }
    }

    fn apply_fonts_and_dark_theme(ctx: &egui::Context) {
        // 1. Load System Fonts if available on macOS
        let mut fonts = FontDefinitions::default();
        if let Ok(sf_bytes) = std::fs::read("/System/Library/Fonts/SFCompact.ttf") {
            fonts.font_data.insert(
                "SFCompact".to_owned(),
                Arc::new(FontData::from_owned(sf_bytes)),
            );
            if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
                family.insert(0, "SFCompact".to_owned());
            }
        }
        if let Ok(mono_bytes) = std::fs::read("/System/Library/Fonts/Monaco.ttf") {
            fonts.font_data.insert(
                "Monaco".to_owned(),
                Arc::new(FontData::from_owned(mono_bytes)),
            );
            if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
                family.insert(0, "Monaco".to_owned());
            }
        }
        ctx.set_fonts(fonts);

        // 2. Configure Dark Visuals
        let mut visuals = egui::Visuals::dark();

        visuals.panel_fill = DarkTheme::SIDEBAR_BG;
        visuals.window_fill = DarkTheme::CARD_BG;
        visuals.extreme_bg_color = DarkTheme::INSET_BG;
        visuals.faint_bg_color = DarkTheme::INSET_BG;

        visuals.selection.bg_fill = Color32::from_rgb(18, 52, 94);
        visuals.selection.stroke = Stroke::new(1.5_f32, DarkTheme::ACCENT_BLUE);

        // Noninteractive widgets
        visuals.widgets.noninteractive.bg_fill = DarkTheme::CARD_BG;
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, DarkTheme::BORDER);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, DarkTheme::TEXT_PRIMARY);

        // Inactive widgets (buttons, inputs)
        visuals.widgets.inactive.bg_fill = DarkTheme::CARD_BG;
        visuals.widgets.inactive.corner_radius = CornerRadius::same(6);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, DarkTheme::BORDER);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, DarkTheme::TEXT_PRIMARY);

        // Hovered widgets
        visuals.widgets.hovered.bg_fill = DarkTheme::CARD_HOVER;
        visuals.widgets.hovered.corner_radius = CornerRadius::same(6);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, DarkTheme::BORDER_LIGHT);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, DarkTheme::TEXT_TITLE);

        // Active widgets
        visuals.widgets.active.bg_fill = Color32::from_rgb(28, 64, 112);
        visuals.widgets.active.corner_radius = CornerRadius::same(6);
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, DarkTheme::ACCENT_BLUE);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, DarkTheme::TEXT_TITLE);

        visuals.widgets.open.bg_fill = DarkTheme::CARD_BG;
        visuals.widgets.open.corner_radius = CornerRadius::same(6);

        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 7.0);
        style.spacing.window_margin = Margin::same(18);
        ctx.set_style(style);
    }

    fn current_center_coordinates(&self) -> (f64, f64) {
        let p = self.map_memory.detached().unwrap_or(self.pos);
        (p.y(), p.x())
    }

    fn jump_to(&mut self, lat: f64, lon: f64) {
        self.pos = lat_lon(lat, lon);
        self.map_memory.center_at(self.pos);
        self.status_message = format!("Centered at {lat:.6}, {lon:.6}");
        self.status_is_error = false;
    }

    fn poll_background_events(&mut self) {
        for event in self.daemon.poll_events() {
            match event {
                DaemonEvent::Ready { locations, routes } => {
                    self.saved_locations = locations;
                    self.gpx_routes = routes;
                    self.status_message = "Connected to iOS Core Daemon".to_string();
                }
                DaemonEvent::DevicesUpdated {
                    devices,
                    selected_udid,
                } => {
                    self.devices = devices;
                    if self.selected_udid.is_none() {
                        self.selected_udid = selected_udid;
                    }
                }
                DaemonEvent::SimulationResult { success, message } => {
                    self.status_message = message;
                    self.status_is_error = !success;
                }
                DaemonEvent::LocationUpdated { latitude, longitude } => {
                    self.jump_to(latitude, longitude);
                }
                DaemonEvent::RouteProgress { current, total } => {
                    self.route_progress_current = current;
                    self.route_progress_total = total;
                }
                DaemonEvent::RouteStateChanged { state } => {
                    self.route_state = state;
                }
                DaemonEvent::SavedLocationsUpdated(locs) => {
                    self.saved_locations = locs;
                }
                DaemonEvent::GpxRoutesUpdated(routes) => {
                    self.gpx_routes = routes;
                }
                DaemonEvent::TunnelStatus(status) => {
                    // Only show password dialog if user requested tunnel or iOS 17+ device exists
                    if status.status.as_deref() == Some("password_required") {
                        self.show_password_modal = true;
                    }
                    if let Some(ref err) = status.error {
                        self.status_message = format!("Tunnel: {err}");
                        self.status_is_error = true;
                    }
                    self.tunnel_status = status;
                }
                DaemonEvent::Error(err) => {
                    self.status_message = format!("Error: {err}");
                    self.status_is_error = true;
                }
            }
        }

        if let Some(results) = self.geocoding.poll_results() {
            self.suggestions = results;
            self.show_suggestions = !self.suggestions.is_empty();
        }
    }
}

fn dirs_cache_dir() -> Option<std::path::PathBuf> {
    if let Some(cache_dir) = dirs::cache_dir() {
        let p = cache_dir.join("LocationSimulator").join("tiles");
        let _ = std::fs::create_dir_all(&p);
        Some(p)
    } else {
        None
    }
}

impl eframe::App for LocationSimulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_background_events();

        // =========================================================================
        // 1. TOP TOOLBAR & SEARCH (Dark macOS Style)
        // =========================================================================
        egui::TopBottomPanel::top("top_toolbar")
            .frame(
                egui::Frame::NONE
                    .fill(DarkTheme::HEADER_BG)
                    .inner_margin(Margin::symmetric(20, 11))
                    .stroke(Stroke::new(1.0_f32, DarkTheme::BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // App Brand with subtle vector target icon
                    ui.horizontal(|ui| {
                        let icon_center = ui.cursor().min + Vec2::new(8.0, 10.0);
                        ui.painter().circle_stroke(
                            icon_center,
                            6.0_f32,
                            Stroke::new(1.5_f32, DarkTheme::ACCENT_BLUE),
                        );
                        ui.painter().circle_filled(
                            icon_center,
                            2.5_f32,
                            DarkTheme::ACCENT_CYAN,
                        );
                        ui.add_space(22.0);

                        ui.label(
                            egui::RichText::new("LOCATION SIMULATOR")
                                .font(FontId::proportional(13.0))
                                .strong()
                                .color(DarkTheme::TEXT_TITLE),
                        );

                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("v1.0")
                                .font(FontId::monospace(10.5))
                                .color(DarkTheme::TEXT_MUTED),
                        );
                    });

                    ui.add_space(24.0);

                    // Dark Spotlight Search Input
                    let search_width = (ui.available_width() - 340.0).max(240.0);
                    let search_frame = egui::Frame::NONE
                        .fill(DarkTheme::INSET_BG)
                        .corner_radius(CornerRadius::same(7))
                        .stroke(Stroke::new(1.0_f32, DarkTheme::BORDER))
                        .inner_margin(Margin::symmetric(12, 5));

                    search_frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Painted vector search glass icon
                            let glass_center = ui.cursor().min + Vec2::new(6.0, 9.0);
                            ui.painter().circle_stroke(
                                glass_center,
                                4.5_f32,
                                Stroke::new(1.2_f32, DarkTheme::TEXT_MUTED),
                            );
                            ui.painter().line_segment(
                                [
                                    glass_center + Vec2::new(3.5, 3.5),
                                    glass_center + Vec2::new(7.0, 7.0),
                                ],
                                Stroke::new(1.5_f32, DarkTheme::TEXT_MUTED),
                            );
                            ui.add_space(18.0);

                            let search_edit = ui.add_sized(
                                [search_width - 65.0, 20.0],
                                egui::TextEdit::singleline(&mut self.search_query)
                                    .hint_text("Search city, address, or enter coordinates (lat, lon)...")
                                    .text_color(DarkTheme::TEXT_PRIMARY)
                                    .frame(false),
                            );

                            if !self.search_query.is_empty() {
                                if ui.small_button("✕").clicked() {
                                    self.search_query.clear();
                                    self.suggestions.clear();
                                    self.show_suggestions = false;
                                }
                            }

                            if search_edit.changed() {
                                let query = self.search_query.trim().to_string();
                                if query.len() >= 2 && query != self.last_searched_query {
                                    self.last_searched_query = query.clone();
                                    self.geocoding.search(query);
                                } else if query.is_empty() {
                                    self.suggestions.clear();
                                    self.show_suggestions = false;
                                }
                            }

                            if search_edit.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                                if let Some((lat, lon)) = parse_coordinates(&self.search_query) {
                                    self.jump_to(lat, lon);
                                    self.show_suggestions = false;
                                } else if let Some(first) = self.suggestions.first() {
                                    self.jump_to(first.lat, first.lon);
                                    self.show_suggestions = false;
                                }
                            }
                        });
                    });

                    // Right Actions
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button(
                                egui::RichText::new("About")
                                    .font(FontId::proportional(12.0))
                                    .color(DarkTheme::TEXT_SECONDARY),
                            )
                            .clicked()
                        {
                            self.show_about_modal = true;
                        }

                        // Tunnel Status Pill
                        if self.tunnel_status.running {
                            let pill = egui::Button::new(
                                egui::RichText::new("● TUNNEL ACTIVE")
                                    .font(FontId::proportional(11.0))
                                    .color(DarkTheme::ACCENT_GREEN)
                                    .strong(),
                            )
                            .fill(DarkTheme::ACCENT_GREEN_BG)
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 120, 60)))
                            .corner_radius(CornerRadius::same(12));

                            if ui.add(pill).on_hover_text("iOS 17+ tunnel is active. Click to stop.").clicked() {
                                self.daemon.stop_tunnel();
                            }
                        } else {
                            let pill = egui::Button::new(
                                egui::RichText::new("○ TUNNEL (iOS 17+)")
                                    .font(FontId::proportional(11.0))
                                    .color(DarkTheme::TEXT_MUTED),
                            )
                            .fill(DarkTheme::CARD_BG)
                            .stroke(Stroke::new(1.0_f32, DarkTheme::BORDER))
                            .corner_radius(CornerRadius::same(12));

                            if ui.add(pill).on_hover_text("Start RemoteXPC tunnel required for iOS 17+ devices").clicked() {
                                self.daemon.start_tunnel();
                                self.status_message = "Starting iOS 17+ tunnel daemon...".to_string();
                            }
                        }
                    });
                });

                // Autocomplete Flyout Window
                if self.show_suggestions && !self.suggestions.is_empty() {
                    ui.add_space(6.0);
                    let flyout = egui::Frame::NONE
                        .fill(DarkTheme::CARD_BG)
                        .corner_radius(CornerRadius::same(8))
                        .stroke(Stroke::new(1.0_f32, DarkTheme::BORDER_LIGHT))
                        .inner_margin(Margin::symmetric(14, 10));

                    flyout.show(ui, |ui| {
                        let mut chosen = None;
                        for item in self.suggestions.iter().take(4) {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("•").color(DarkTheme::ACCENT_CYAN));
                                let text = if item.display_name.len() > 70 {
                                    format!("{}...", &item.display_name[..70])
                                } else {
                                    item.display_name.clone()
                                };
                                if ui
                                    .link(
                                        egui::RichText::new(text)
                                            .font(FontId::proportional(12.5))
                                            .color(DarkTheme::TEXT_PRIMARY),
                                    )
                                    .clicked()
                                {
                                    chosen = Some((item.lat, item.lon));
                                }
                            });
                            ui.add_space(2.0);
                        }
                        if let Some((lat, lon)) = chosen {
                            self.jump_to(lat, lon);
                            self.show_suggestions = false;
                        }
                    });
                }
            });

        // =========================================================================
        // 2. BOTTOM CONTROL & STATUS BAR
        // =========================================================================
        egui::TopBottomPanel::bottom("bottom_bar")
            .frame(
                egui::Frame::NONE
                    .fill(DarkTheme::BOTTOM_BG)
                    .inner_margin(Margin::symmetric(22, 12))
                    .stroke(Stroke::new(1.0_f32, DarkTheme::BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Status dot & text
                    let dot_color = if self.status_is_error {
                        DarkTheme::ACCENT_RED
                    } else if self.status_message.starts_with("Setting") || self.status_message.starts_with("Starting") {
                        DarkTheme::ACCENT_AMBER
                    } else {
                        DarkTheme::ACCENT_GREEN
                    };

                    let dot_center = ui.cursor().min + Vec2::new(4.0, 9.0);
                    ui.painter().circle_filled(dot_center, 3.5_f32, dot_color);
                    ui.add_space(14.0);

                    ui.label(
                        egui::RichText::new(&self.status_message)
                            .font(FontId::proportional(12.5))
                            .color(if self.status_is_error { DarkTheme::ACCENT_RED } else { DarkTheme::TEXT_SECONDARY }),
                    );

                    // Right Side Action Buttons
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let (cur_lat, cur_lon) = self.current_center_coordinates();
                        let has_device = self.selected_udid.is_some() || !self.devices.is_empty();

                        // Primary Action: "Simulate Location" (Electric Apple Blue)
                        let set_btn = egui::Button::new(
                            egui::RichText::new("Simulate Location")
                                .font(FontId::proportional(13.5))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(if has_device {
                            DarkTheme::ACCENT_BLUE
                        } else {
                            Color32::from_rgb(40, 50, 70)
                        })
                        .corner_radius(CornerRadius::same(6));

                        if ui
                            .add_enabled(has_device, set_btn)
                            .on_hover_text("Send simulated GPS coordinates to selected device")
                            .clicked()
                        {
                            self.daemon.set_location(self.selected_udid.as_deref(), cur_lat, cur_lon);
                            self.status_message = format!("Simulating coordinates: {cur_lat:.5}, {cur_lon:.5}...");
                            self.status_is_error = false;
                        }

                        // Clear Button: Subtle dark red outline
                        let clear_btn = egui::Button::new(
                            egui::RichText::new("Clear Location")
                                .font(FontId::proportional(12.5))
                                .color(DarkTheme::ACCENT_RED),
                        )
                        .fill(DarkTheme::ACCENT_RED_BG)
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(120, 30, 40)))
                        .corner_radius(CornerRadius::same(6));

                        if ui
                            .add_enabled(has_device, clear_btn)
                            .on_hover_text("Reset device to actual physical GPS location")
                            .clicked()
                        {
                            self.daemon.clear_location(self.selected_udid.as_deref());
                            self.status_message = "Restoring device hardware GPS...".to_string();
                            self.status_is_error = false;
                        }

                        // Save Bookmark Button
                        let save_btn = egui::Button::new(
                            egui::RichText::new("Save Bookmark")
                                .font(FontId::proportional(12.5))
                                .color(DarkTheme::TEXT_PRIMARY),
                        )
                        .fill(DarkTheme::CARD_BG)
                        .stroke(Stroke::new(1.0_f32, DarkTheme::BORDER))
                        .corner_radius(CornerRadius::same(6));

                        if ui.add(save_btn).clicked() {
                            self.save_name_input = format!("Location ({cur_lat:.4}, {cur_lon:.4})");
                            self.show_save_modal = true;
                        }
                    });
                });
            });

        // =========================================================================
        // 3. LEFT SIDEBAR (Dark Cards & Segmented Tabs)
        // =========================================================================
        egui::SidePanel::left("left_sidebar")
            .exact_width(295.0)
            .frame(
                egui::Frame::NONE
                    .fill(DarkTheme::SIDEBAR_BG)
                    .inner_margin(Margin::symmetric(14, 14))
                    .stroke(Stroke::new(1.0_f32, DarkTheme::BORDER)),
            )
            .show(ctx, |ui| {
                // Section Header: Devices
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("CONNECTED DEVICES")
                            .font(FontId::proportional(11.0))
                            .strong()
                            .color(DarkTheme::TEXT_MUTED),
                    );

                    ui.label(
                        egui::RichText::new(format!("({})", self.devices.len()))
                            .font(FontId::monospace(10.5))
                            .color(DarkTheme::TEXT_SECONDARY),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button(
                                egui::RichText::new("↻ Refresh")
                                    .font(FontId::proportional(11.0))
                                    .color(DarkTheme::TEXT_CYAN),
                            )
                            .clicked()
                        {
                            self.daemon.refresh_devices();
                        }
                    });
                });

                ui.add_space(8.0);

                // Devices List
                if self.devices.is_empty() {
                    let empty = egui::Frame::NONE
                        .fill(DarkTheme::CARD_BG)
                        .corner_radius(CornerRadius::same(8))
                        .stroke(Stroke::new(1.0_f32, DarkTheme::BORDER))
                        .inner_margin(Margin::symmetric(16, 18));

                    empty.show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            // Painted vector phone shape
                            let phone_rect = Rect::from_center_size(
                                ui.cursor().min + Vec2::new(ui.available_width() / 2.0, 14.0),
                                Vec2::new(18.0, 26.0),
                            );
                            ui.painter().rect_stroke(
                                phone_rect,
                                CornerRadius::same(4),
                                Stroke::new(1.5_f32, DarkTheme::TEXT_MUTED),
                                StrokeKind::Outside,
                            );
                            ui.painter().circle_filled(
                                Pos2::new(phone_rect.center().x, phone_rect.bottom() - 4.0),
                                1.5_f32,
                                DarkTheme::TEXT_MUTED,
                            );
                            ui.add_space(34.0);

                            ui.label(
                                egui::RichText::new("No iOS Device Found")
                                    .font(FontId::proportional(13.0))
                                    .strong()
                                    .color(DarkTheme::TEXT_PRIMARY),
                            );
                            ui.add_space(2.0);
                            ui.label(
                                egui::RichText::new("Connect via USB cable and trust this computer on your device.")
                                    .font(FontId::proportional(11.0))
                                    .color(DarkTheme::TEXT_MUTED),
                            );
                        });
                    });
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(160.0)
                        .id_salt("devices_list_scroll")
                        .show(ui, |ui| {
                            for dev in &self.devices {
                                let is_selected = self.selected_udid.as_deref() == Some(&dev.udid);

                                let (card_bg, border_color, border_w) = if is_selected {
                                    (
                                        DarkTheme::CARD_SELECTED,
                                        DarkTheme::BORDER_ACTIVE,
                                        1.5_f32,
                                    )
                                } else {
                                    (
                                        DarkTheme::CARD_BG,
                                        DarkTheme::BORDER,
                                        1.0_f32,
                                    )
                                };

                                let card = egui::Frame::NONE
                                    .fill(card_bg)
                                    .corner_radius(CornerRadius::same(8))
                                    .stroke(Stroke::new(border_w, border_color))
                                    .inner_margin(Margin::symmetric(12, 10));

                                card.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Left Device Avatar Badge
                                        let badge_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(32.0, 32.0));
                                        ui.painter().rect_filled(
                                            badge_rect,
                                            CornerRadius::same(6),
                                            if is_selected { Color32::from_rgb(16, 40, 72) } else { DarkTheme::INSET_BG },
                                        );
                                        ui.painter().text(
                                            badge_rect.center(),
                                            Align2::CENTER_CENTER,
                                            "iOS",
                                            FontId::monospace(10.5),
                                            if is_selected { DarkTheme::TEXT_CYAN } else { DarkTheme::TEXT_MUTED },
                                        );
                                        ui.add_space(38.0);

                                        // Device Info Stack
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                let name_label = egui::RichText::new(&dev.name)
                                                    .font(FontId::proportional(13.0))
                                                    .strong()
                                                    .color(DarkTheme::TEXT_TITLE);

                                                if ui.selectable_label(is_selected, name_label).clicked() {
                                                    self.selected_udid = Some(dev.udid.clone());
                                                    self.daemon.select_device(&dev.udid);
                                                }
                                            });

                                            ui.label(
                                                egui::RichText::new(format!("{} · iOS {}", dev.model, dev.ios_version))
                                                    .font(FontId::proportional(11.0))
                                                    .color(DarkTheme::TEXT_SECONDARY),
                                            );

                                            ui.label(
                                                egui::RichText::new(format!("● Connected ({})", dev.connection_type))
                                                    .font(FontId::proportional(10.5))
                                                    .color(DarkTheme::ACCENT_GREEN),
                                            );
                                        });
                                    });
                                });
                                ui.add_space(6.0);
                            }
                        });
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                // Segmented Tab Switcher (Dark Inset Container)
                let tab_container = egui::Frame::NONE
                    .fill(DarkTheme::INSET_BG)
                    .corner_radius(CornerRadius::same(7))
                    .stroke(Stroke::new(1.0_f32, DarkTheme::BORDER))
                    .inner_margin(Margin::symmetric(3, 3));

                tab_container.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let btn_width = (ui.available_width() - 8.0) / 2.0;

                        let is_loc = self.active_tab == SidebarTab::Locations;
                        let loc_btn = egui::Button::new(
                            egui::RichText::new("Saved Places")
                                .font(FontId::proportional(12.0))
                                .strong()
                                .color(if is_loc { DarkTheme::TEXT_TITLE } else { DarkTheme::TEXT_MUTED }),
                        )
                        .fill(if is_loc { DarkTheme::CARD_BG } else { Color32::TRANSPARENT })
                        .stroke(if is_loc {
                            Stroke::new(1.0_f32, DarkTheme::BORDER_LIGHT)
                        } else {
                            Stroke::NONE
                        })
                        .corner_radius(CornerRadius::same(5));

                        if ui.add_sized([btn_width, 24.0], loc_btn).clicked() {
                            self.active_tab = SidebarTab::Locations;
                        }

                        let is_routes = self.active_tab == SidebarTab::Routes;
                        let routes_btn = egui::Button::new(
                            egui::RichText::new("GPX Tracks")
                                .font(FontId::proportional(12.0))
                                .strong()
                                .color(if is_routes { DarkTheme::TEXT_TITLE } else { DarkTheme::TEXT_MUTED }),
                        )
                        .fill(if is_routes { DarkTheme::CARD_BG } else { Color32::TRANSPARENT })
                        .stroke(if is_routes {
                            Stroke::new(1.0_f32, DarkTheme::BORDER_LIGHT)
                        } else {
                            Stroke::NONE
                        })
                        .corner_radius(CornerRadius::same(5));

                        if ui.add_sized([btn_width, 24.0], routes_btn).clicked() {
                            self.active_tab = SidebarTab::Routes;
                        }
                    });
                });

                ui.add_space(8.0);

                // TAB 1: SAVED PLACES
                if self.active_tab == SidebarTab::Locations {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("BOOKMARKS ({})", self.saved_locations.len()))
                                .font(FontId::proportional(11.0))
                                .strong()
                                .color(DarkTheme::TEXT_MUTED),
                        );

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .button(
                                    egui::RichText::new("+ Save Current")
                                        .font(FontId::proportional(11.0))
                                        .color(DarkTheme::TEXT_CYAN),
                                )
                                .clicked()
                            {
                                let (lat, lon) = self.current_center_coordinates();
                                self.save_name_input = format!("Location ({lat:.4}, {lon:.4})");
                                self.show_save_modal = true;
                            }
                        });
                    });

                    ui.add_space(6.0);

                    egui::ScrollArea::vertical()
                        .id_salt("places_list_scroll")
                        .show(ui, |ui| {
                            let mut loc_to_remove = None;
                            let mut loc_to_jump = None;
                            let mut loc_to_set = None;

                            for loc in &self.saved_locations {
                                let row = egui::Frame::NONE
                                    .fill(DarkTheme::CARD_BG)
                                    .corner_radius(CornerRadius::same(7))
                                    .stroke(Stroke::new(1.0_f32, DarkTheme::BORDER))
                                    .inner_margin(Margin::symmetric(12, 8));

                                row.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Category color dot
                                        let cat_color = match loc.category.as_str() {
                                            "work" => DarkTheme::ACCENT_BLUE,
                                            "home" => Color32::from_rgb(168, 85, 247),
                                            _ => DarkTheme::TEXT_CYAN,
                                        };

                                        let dot_pos = ui.cursor().min + Vec2::new(3.0, 8.0);
                                        ui.painter().circle_filled(dot_pos, 3.0_f32, cat_color);
                                        ui.add_space(12.0);

                                        ui.vertical(|ui| {
                                            if ui
                                                .link(
                                                    egui::RichText::new(&loc.name)
                                                        .font(FontId::proportional(12.5))
                                                        .strong()
                                                        .color(DarkTheme::TEXT_PRIMARY),
                                                )
                                                .clicked()
                                            {
                                                loc_to_jump = Some((loc.latitude, loc.longitude));
                                            }

                                            ui.label(
                                                egui::RichText::new(format!("{:.4}, {:.4}", loc.latitude, loc.longitude))
                                                    .font(FontId::monospace(10.0))
                                                    .color(DarkTheme::TEXT_MUTED),
                                            );
                                        });

                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            if ui.small_button("✕").on_hover_text("Delete bookmark").clicked() {
                                                loc_to_remove = Some(loc.id.clone());
                                            }
                                            if ui
                                                .button(
                                                    egui::RichText::new("Set")
                                                        .font(FontId::proportional(11.0))
                                                        .color(DarkTheme::TEXT_CYAN),
                                                )
                                                .on_hover_text("Teleport directly to this place")
                                                .clicked()
                                            {
                                                loc_to_set = Some((loc.latitude, loc.longitude));
                                            }
                                        });
                                    });
                                });
                                ui.add_space(4.0);
                            }

                            if let Some((lat, lon)) = loc_to_jump {
                                self.jump_to(lat, lon);
                            }
                            if let Some((lat, lon)) = loc_to_set {
                                self.jump_to(lat, lon);
                                self.daemon.set_location(self.selected_udid.as_deref(), lat, lon);
                            }
                            if let Some(id) = loc_to_remove {
                                self.daemon.remove_saved_location(&id);
                            }
                        });
                }
                // TAB 2: GPX TRACKS
                else {
                    let import_btn = egui::Button::new(
                        egui::RichText::new("Import GPX Route...")
                            .font(FontId::proportional(12.5))
                            .color(DarkTheme::TEXT_CYAN)
                            .strong(),
                    )
                    .fill(Color32::from_rgb(18, 36, 56))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(30, 60, 96)))
                    .corner_radius(CornerRadius::same(6));

                    if ui.add_sized([ui.available_width(), 30.0], import_btn).clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("GPX Routes", &["gpx"])
                            .pick_file()
                        {
                            if let Some(path_str) = path.to_str() {
                                self.daemon.import_gpx(path_str);
                                self.status_message = format!("Imported route: {}", path.file_name().unwrap_or_default().to_string_lossy());
                            }
                        }
                    }

                    ui.add_space(8.0);

                    egui::ScrollArea::vertical()
                        .id_salt("routes_list_scroll")
                        .show(ui, |ui| {
                            let mut route_to_remove = None;
                            let mut route_to_start = None;

                            for route in &self.gpx_routes {
                                let card = egui::Frame::NONE
                                    .fill(DarkTheme::CARD_BG)
                                    .corner_radius(CornerRadius::same(7))
                                    .stroke(Stroke::new(1.0_f32, DarkTheme::BORDER))
                                    .inner_margin(Margin::symmetric(12, 10));

                                card.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new(&route.name)
                                                    .font(FontId::proportional(12.5))
                                                    .strong()
                                                    .color(DarkTheme::TEXT_PRIMARY),
                                            );
                                            ui.label(
                                                egui::RichText::new(format!("{} waypoints", route.waypoints.len()))
                                                    .font(FontId::proportional(10.5))
                                                    .color(DarkTheme::TEXT_MUTED),
                                            );
                                        });

                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            if ui.small_button("✕").clicked() {
                                                route_to_remove = Some(route.id.clone());
                                            }
                                            if ui
                                                .button(
                                                    egui::RichText::new("Play")
                                                        .font(FontId::proportional(11.0))
                                                        .color(DarkTheme::ACCENT_BLUE),
                                                )
                                                .clicked()
                                            {
                                                route_to_start = Some(route.id.clone());
                                            }
                                        });
                                    });
                                });
                                ui.add_space(4.0);
                            }

                            if let Some(id) = route_to_remove {
                                self.daemon.remove_gpx_route(&id);
                            }
                            if let Some(id) = route_to_start {
                                self.daemon.start_route(&id, self.route_speed, self.route_loop);
                                self.status_message = "Starting route simulation...".to_string();
                            }

                            // Active Route Simulation Console
                            if self.route_state != "idle" {
                                ui.add_space(8.0);
                                let console = egui::Frame::NONE
                                    .fill(Color32::from_rgb(20, 26, 38))
                                    .corner_radius(CornerRadius::same(8))
                                    .stroke(Stroke::new(1.5_f32, DarkTheme::ACCENT_BLUE))
                                    .inner_margin(Margin::symmetric(14, 12));

                                console.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new("ROUTE SIMULATION")
                                                .font(FontId::proportional(10.0))
                                                .strong()
                                                .color(DarkTheme::TEXT_MUTED),
                                        );
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            let (badge_text, badge_color) = if self.route_state == "running" {
                                                ("● RUNNING", DarkTheme::ACCENT_GREEN)
                                            } else {
                                                ("❚❚ PAUSED", DarkTheme::ACCENT_AMBER)
                                            };
                                            ui.label(
                                                egui::RichText::new(badge_text)
                                                    .font(FontId::proportional(10.0))
                                                    .strong()
                                                    .color(badge_color),
                                            );
                                        });
                                    });

                                    ui.add_space(4.0);

                                    let progress = if self.route_progress_total > 0 {
                                        self.route_progress_current as f32 / self.route_progress_total as f32
                                    } else {
                                        0.0
                                    };
                                    ui.add(egui::ProgressBar::new(progress).show_percentage());

                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Waypoint {} / {}",
                                            self.route_progress_current, self.route_progress_total
                                        ))
                                        .font(FontId::proportional(10.5))
                                        .color(DarkTheme::TEXT_SECONDARY),
                                    );

                                    ui.add_space(6.0);

                                    ui.horizontal(|ui| {
                                        if self.route_state == "running" {
                                            if ui.button("Pause").clicked() {
                                                self.daemon.pause_route();
                                            }
                                        } else if self.route_state == "paused" {
                                            if ui.button("Resume").clicked() {
                                                self.daemon.resume_route();
                                            }
                                        }
                                        if ui.button("Stop").clicked() {
                                            self.daemon.stop_route();
                                        }
                                    });
                                });
                            }
                        });
                }
            });

        // =========================================================================
        // 4. MAIN MAP CANVAS (CartoDB Dark Matter + Precision Cyan Reticle)
        // =========================================================================
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(DarkTheme::WINDOW_BG))
            .show(ctx, |ui| {
                let map = Map::new(Some(&mut self.tiles), &mut self.map_memory, self.pos);
                let response = ui.add(map);

                let painter = ui.painter_at(response.rect);
                let center = response.rect.center();

                // Avionics Precision Reticle (Electric Cyan #38BDF8)
                let reticle_color = Color32::from_rgba_unmultiplied(56, 189, 248, 240);
                let reticle_stroke = Stroke::new(1.5_f32, reticle_color);

                // 4-Quadrant Crosshair Ticks
                painter.line_segment(
                    [
                        Pos2::new(center.x - 16.0, center.y),
                        Pos2::new(center.x - 5.0, center.y),
                    ],
                    reticle_stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x + 5.0, center.y),
                        Pos2::new(center.x + 16.0, center.y),
                    ],
                    reticle_stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x, center.y - 16.0),
                        Pos2::new(center.x, center.y - 5.0),
                    ],
                    reticle_stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x, center.y + 5.0),
                        Pos2::new(center.x, center.y + 16.0),
                    ],
                    reticle_stroke,
                );

                // Center Reticle Ring & Dot
                painter.circle_stroke(center, 4.5_f32, reticle_stroke);
                painter.circle_filled(center, 1.5_f32, reticle_color);

                // Floating Glassmorphic Coordinates Pill (Dark Frosted Glass)
                let (lat, lon) = self.current_center_coordinates();
                let hud_pos = Pos2::new(response.rect.left() + 20.0, response.rect.bottom() - 44.0);
                let hud_rect = Rect::from_min_size(hud_pos, Vec2::new(230.0, 30.0));

                painter.rect_filled(
                    hud_rect,
                    CornerRadius::same(7),
                    Color32::from_rgba_unmultiplied(17, 20, 27, 235),
                );
                painter.rect_stroke(
                    hud_rect,
                    CornerRadius::same(7),
                    Stroke::new(1.0_f32, DarkTheme::BORDER_LIGHT),
                    StrokeKind::Outside,
                );
                painter.text(
                    hud_rect.center(),
                    Align2::CENTER_CENTER,
                    format!("{lat:.6}°, {lon:.6}°"),
                    FontId::monospace(12.0),
                    DarkTheme::TEXT_CYAN,
                );

                // Floating Zoom Controls (+ / −)
                let zoom_top_pos = Pos2::new(response.rect.right() - 46.0, response.rect.top() + 18.0);
                let zoom_in_rect = Rect::from_min_size(zoom_top_pos, Vec2::new(30.0, 30.0));
                let zoom_out_rect = Rect::from_min_size(zoom_top_pos + Vec2::new(0.0, 34.0), Vec2::new(30.0, 30.0));

                painter.rect_filled(zoom_in_rect, CornerRadius::same(5), DarkTheme::CARD_BG);
                painter.rect_stroke(zoom_in_rect, CornerRadius::same(5), Stroke::new(1.0_f32, DarkTheme::BORDER), StrokeKind::Outside);
                painter.text(zoom_in_rect.center(), Align2::CENTER_CENTER, "+", FontId::proportional(17.0), DarkTheme::TEXT_PRIMARY);

                painter.rect_filled(zoom_out_rect, CornerRadius::same(5), DarkTheme::CARD_BG);
                painter.rect_stroke(zoom_out_rect, CornerRadius::same(5), Stroke::new(1.0_f32, DarkTheme::BORDER), StrokeKind::Outside);
                painter.text(zoom_out_rect.center(), Align2::CENTER_CENTER, "−", FontId::proportional(17.0), DarkTheme::TEXT_PRIMARY);

                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        if zoom_in_rect.contains(pos) {
                            let _ = self.map_memory.zoom_in();
                        } else if zoom_out_rect.contains(pos) {
                            let _ = self.map_memory.zoom_out();
                        }
                    }
                }
            });

        // =========================================================================
        // 5. DARK MODAL WINDOWS
        // =========================================================================

        if self.show_password_modal {
            egui::Window::new("Admin Privileges Required")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new("iOS 17+ devices require root permissions to initialize the RemoteXPC tunnel daemon.")
                            .color(DarkTheme::TEXT_SECONDARY),
                    );
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Password:").color(DarkTheme::TEXT_PRIMARY));
                    ui.add(egui::TextEdit::singleline(&mut self.password_input).password(true));

                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        let auth_btn = egui::Button::new(
                            egui::RichText::new("Authenticate")
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(DarkTheme::ACCENT_BLUE)
                        .corner_radius(CornerRadius::same(6));

                        if ui.add(auth_btn).clicked() {
                            self.daemon.submit_tunnel_password(&self.password_input);
                            self.password_input.clear();
                            self.show_password_modal = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.daemon.cancel_tunnel_password();
                            self.password_input.clear();
                            self.show_password_modal = false;
                        }
                    });
                });
        }

        if self.show_save_modal {
            egui::Window::new("Save Location Bookmark")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Bookmark Name:").color(DarkTheme::TEXT_PRIMARY));
                    ui.text_edit_singleline(&mut self.save_name_input);

                    ui.add_space(6.0);
                    let (lat, lon) = self.current_center_coordinates();
                    ui.label(
                        egui::RichText::new(format!("Target: {lat:.6}°, {lon:.6}°"))
                            .font(FontId::monospace(11.0))
                            .color(DarkTheme::TEXT_CYAN),
                    );

                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        let save_btn = egui::Button::new(
                            egui::RichText::new("Save")
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(DarkTheme::ACCENT_BLUE)
                        .corner_radius(CornerRadius::same(6));

                        if ui.add(save_btn).clicked() {
                            if !self.save_name_input.trim().is_empty() {
                                self.daemon.add_saved_location(
                                    &self.save_name_input,
                                    lat,
                                    lon,
                                    &self.save_category_input,
                                );
                                self.status_message = format!("Saved bookmark: '{}'", self.save_name_input);
                                self.show_save_modal = false;
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_save_modal = false;
                        }
                    });
                });
        }

        if self.show_about_modal {
            egui::Window::new("About Location Simulator")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new("Location Simulator")
                            .font(FontId::proportional(16.0))
                            .strong()
                            .color(DarkTheme::TEXT_TITLE),
                    );
                    ui.label(
                        egui::RichText::new("Native Rust Edition · Dark Mode")
                            .font(FontId::proportional(12.0))
                            .color(DarkTheme::TEXT_CYAN),
                    );
                    ui.add_space(10.0);
                    ui.label("High-performance iOS Location Simulation tool.");
                    ui.label("• Rendering: Pure Rust GPU (Metal/OpenGL)");
                    ui.label("• Map Tiles: CartoDB Dark Matter");
                    ui.label("• Device Protocol: pymobiledevice3 async daemon");
                    ui.label("• iOS Support: iOS 12.0 – iOS 18+ (RemoteXPC Tunnel)");
                    ui.add_space(14.0);
                    if ui.button("Close").clicked() {
                        self.show_about_modal = false;
                    }
                });
        }
    }
}
