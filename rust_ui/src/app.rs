use crate::daemon::{DaemonClient, DaemonEvent};
use crate::geocoding::{parse_coordinates, GeocodingClient};
use crate::models::{GPXRoute, SavedLocation, SearchSuggestion, TunnelStatus, iOSDevice};
use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Layout, Margin, Pos2, Rect, Stroke,
    StrokeKind, Vec2,
};
use walkers::{lat_lon, HttpOptions, HttpTiles, Map, MapMemory, Position};

#[derive(PartialEq, Clone, Copy)]
enum SidebarTab {
    Locations,
    Routes,
}

// ---------------------------------------------------------------------------
// Design System Tokens (macOS Human Interface Guidelines inspired)
// ---------------------------------------------------------------------------
#[allow(dead_code)]
struct Theme;
#[allow(dead_code)]
impl Theme {
    // Backgrounds
    const APP_BG: Color32 = Color32::from_rgb(246, 248, 250);
    const SIDEBAR_BG: Color32 = Color32::from_rgb(255, 255, 255);
    const SURFACE: Color32 = Color32::from_rgb(255, 255, 255);
    const SURFACE_HOVER: Color32 = Color32::from_rgb(248, 250, 252);
    const SURFACE_MUTED: Color32 = Color32::from_rgb(241, 245, 249);

    // Borders & Dividers
    const BORDER_SUBTLE: Color32 = Color32::from_rgb(226, 232, 240);
    const BORDER_HOVER: Color32 = Color32::from_rgb(203, 213, 225);

    // Text hierarchy
    const TEXT_PRIMARY: Color32 = Color32::from_rgb(15, 23, 42); // Slate 900
    const TEXT_SECONDARY: Color32 = Color32::from_rgb(71, 85, 105); // Slate 600
    const TEXT_MUTED: Color32 = Color32::from_rgb(148, 163, 184); // Slate 400

    // Accent Colors
    const ACCENT_BLUE: Color32 = Color32::from_rgb(0, 113, 227); // Apple System Blue
    const ACCENT_BLUE_HOVER: Color32 = Color32::from_rgb(0, 93, 190);
    const ACCENT_BLUE_BG: Color32 = Color32::from_rgb(238, 246, 255);

    const SUCCESS_GREEN: Color32 = Color32::from_rgb(40, 167, 69);
    const SUCCESS_BG: Color32 = Color32::from_rgb(236, 253, 243);

    const DANGER_RED: Color32 = Color32::from_rgb(220, 38, 38);
    const DANGER_BG: Color32 = Color32::from_rgb(254, 242, 242);

    const WARNING_AMBER: Color32 = Color32::from_rgb(217, 119, 6);
    const WARNING_BG: Color32 = Color32::from_rgb(254, 243, 199);
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
        Self::apply_macos_theme(&cc.egui_ctx);

        let daemon = DaemonClient::spawn().expect("Failed to initialize Python daemon sidecar");
        let geocoding = GeocodingClient::new();

        let http_options = HttpOptions {
            cache: dirs_cache_dir(),
            ..Default::default()
        };
        let tiles = HttpTiles::with_options(
            walkers::sources::OpenStreetMap,
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

    fn apply_macos_theme(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::light();

        visuals.panel_fill = Theme::SIDEBAR_BG;
        visuals.window_fill = Theme::SURFACE;
        visuals.extreme_bg_color = Theme::SURFACE;
        visuals.faint_bg_color = Theme::SURFACE_MUTED;

        visuals.selection.bg_fill = Theme::ACCENT_BLUE_BG;
        visuals.selection.stroke = Stroke::new(1.5_f32, Theme::ACCENT_BLUE);

        visuals.widgets.noninteractive.bg_fill = Theme::SURFACE;
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Theme::BORDER_SUBTLE);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Theme::TEXT_PRIMARY);

        visuals.widgets.inactive.bg_fill = Theme::SURFACE;
        visuals.widgets.inactive.corner_radius = CornerRadius::same(6);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Theme::BORDER_SUBTLE);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Theme::TEXT_SECONDARY);

        visuals.widgets.hovered.bg_fill = Theme::SURFACE_HOVER;
        visuals.widgets.hovered.corner_radius = CornerRadius::same(6);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Theme::BORDER_HOVER);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Theme::TEXT_PRIMARY);

        visuals.widgets.active.bg_fill = Theme::ACCENT_BLUE_BG;
        visuals.widgets.active.corner_radius = CornerRadius::same(6);
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, Theme::ACCENT_BLUE);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Theme::ACCENT_BLUE);

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
                    if status.status.as_deref() == Some("password_required") {
                        self.show_password_modal = true;
                    }
                    if let Some(ref err) = status.error {
                        self.status_message = format!("Tunnel error: {err}");
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
        // 1. TOP TOOLBAR & SPOTLIGHT SEARCH BAR
        // =========================================================================
        egui::TopBottomPanel::top("top_toolbar")
            .frame(
                egui::Frame::NONE
                    .fill(Theme::SURFACE)
                    .inner_margin(Margin::symmetric(20, 10))
                    .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Clean Brand Badge
                    ui.horizontal(|ui| {
                        let dot_rect = Rect::from_min_size(ui.cursor().min + Vec2::new(0.0, 4.0), Vec2::new(8.0, 8.0));
                        ui.painter().circle_filled(dot_rect.center(), 4.0_f32, Theme::ACCENT_BLUE);
                        ui.add_space(14.0);

                        ui.label(
                            egui::RichText::new("Location Simulator")
                                .font(FontId::proportional(15.0))
                                .strong()
                                .color(Theme::TEXT_PRIMARY),
                        );
                    });

                    ui.add_space(20.0);

                    // Professional Spotlight Search Pill
                    let search_width = (ui.available_width() - 320.0).max(240.0);
                    let search_frame = egui::Frame::NONE
                        .fill(Theme::SURFACE_MUTED)
                        .corner_radius(CornerRadius::same(7))
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .inner_margin(Margin::symmetric(12, 5));

                    search_frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("⌕")
                                    .font(FontId::proportional(16.0))
                                    .color(Theme::TEXT_MUTED),
                            );

                            let search_edit = ui.add_sized(
                                [search_width - 55.0, 22.0],
                                egui::TextEdit::singleline(&mut self.search_query)
                                    .hint_text("Search location, city, coordinates (e.g. 37.77, -122.41)...")
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

                    // Right Utility Actions
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // About Menu button
                        if ui
                            .button(
                                egui::RichText::new("About")
                                    .font(FontId::proportional(12.5))
                                    .color(Theme::TEXT_SECONDARY),
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
                                    .color(Theme::SUCCESS_GREEN)
                                    .strong(),
                            )
                            .fill(Theme::SUCCESS_BG)
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(187, 247, 208)))
                            .corner_radius(CornerRadius::same(12));

                            if ui.add(pill).on_hover_text("Tunnel is running. Click to stop.").clicked() {
                                self.daemon.stop_tunnel();
                            }
                        } else {
                            let pill = egui::Button::new(
                                egui::RichText::new("○ TUNNEL (iOS 17+)")
                                    .font(FontId::proportional(11.0))
                                    .color(Theme::TEXT_MUTED),
                            )
                            .fill(Theme::SURFACE_MUTED)
                            .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
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
                        .fill(Theme::SURFACE)
                        .corner_radius(CornerRadius::same(8))
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .inner_margin(Margin::symmetric(14, 10));

                    flyout.show(ui, |ui| {
                        let mut chosen = None;
                        for item in self.suggestions.iter().take(4) {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("📍").color(Theme::ACCENT_BLUE));
                                let text = if item.display_name.len() > 70 {
                                    format!("{}...", &item.display_name[..70])
                                } else {
                                    item.display_name.clone()
                                };
                                if ui
                                    .link(
                                        egui::RichText::new(text)
                                            .font(FontId::proportional(13.0))
                                            .color(Theme::TEXT_PRIMARY),
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
        // 2. BOTTOM CONTROL & SIMULATION STATUS BAR
        // =========================================================================
        egui::TopBottomPanel::bottom("bottom_bar")
            .frame(
                egui::Frame::NONE
                    .fill(Theme::SURFACE)
                    .inner_margin(Margin::symmetric(24, 12))
                    .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Live Status Dot & Message
                    let (dot_color, text_color) = if self.status_is_error {
                        (Theme::DANGER_RED, Theme::DANGER_RED)
                    } else if self.status_message.starts_with("Setting") || self.status_message.starts_with("Starting") {
                        (Theme::WARNING_AMBER, Theme::WARNING_AMBER)
                    } else {
                        (Theme::SUCCESS_GREEN, Theme::TEXT_SECONDARY)
                    };

                    ui.label(egui::RichText::new("●").color(dot_color).font(FontId::proportional(11.0)));
                    ui.label(
                        egui::RichText::new(&self.status_message)
                            .font(FontId::proportional(13.0))
                            .color(text_color),
                    );

                    // Right Side Action Buttons
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let (cur_lat, cur_lon) = self.current_center_coordinates();
                        let has_device = self.selected_udid.is_some() || !self.devices.is_empty();

                        // Primary Action: "Set Location" (Apple Blue, clean bold font)
                        let set_btn = egui::Button::new(
                            egui::RichText::new("Simulate Location")
                                .font(FontId::proportional(13.5))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(if has_device {
                            Theme::ACCENT_BLUE
                        } else {
                            Color32::from_rgb(203, 213, 225)
                        })
                        .corner_radius(CornerRadius::same(6));

                        if ui
                            .add_enabled(has_device, set_btn)
                            .on_hover_text("Set current map crosshair location on selected iOS device")
                            .clicked()
                        {
                            self.daemon.set_location(self.selected_udid.as_deref(), cur_lat, cur_lon);
                            self.status_message = format!("Simulating position: {cur_lat:.5}, {cur_lon:.5}...");
                            self.status_is_error = false;
                        }

                        // Clear Button: Clean subtle red outline
                        let clear_btn = egui::Button::new(
                            egui::RichText::new("Clear Location")
                                .font(FontId::proportional(13.0))
                                .color(Theme::DANGER_RED),
                        )
                        .fill(Theme::DANGER_BG)
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(254, 202, 202)))
                        .corner_radius(CornerRadius::same(6));

                        if ui
                            .add_enabled(has_device, clear_btn)
                            .on_hover_text("Clear location simulation and restore real device GPS")
                            .clicked()
                        {
                            self.daemon.clear_location(self.selected_udid.as_deref());
                            self.status_message = "Restoring hardware GPS location...".to_string();
                            self.status_is_error = false;
                        }

                        // Save Bookmark Button: Clean neutral
                        let save_btn = egui::Button::new(
                            egui::RichText::new("Save Bookmark")
                                .font(FontId::proportional(13.0))
                                .color(Theme::TEXT_PRIMARY),
                        )
                        .fill(Theme::SURFACE_MUTED)
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .corner_radius(CornerRadius::same(6));

                        if ui.add(save_btn).clicked() {
                            self.save_name_input = format!("Location ({cur_lat:.4}, {cur_lon:.4})");
                            self.show_save_modal = true;
                        }
                    });
                });
            });

        // =========================================================================
        // 3. LEFT SIDEBAR (Devices & Navigation Panel)
        // =========================================================================
        egui::SidePanel::left("left_sidebar")
            .exact_width(300.0)
            .frame(
                egui::Frame::NONE
                    .fill(Theme::APP_BG)
                    .inner_margin(Margin::symmetric(14, 14))
                    .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE)),
            )
            .show(ctx, |ui| {
                // Section Header: Devices
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("CONNECTED DEVICES")
                            .font(FontId::proportional(11.0))
                            .strong()
                            .color(Theme::TEXT_MUTED),
                    );

                    let count_badge = format!("{}", self.devices.len());
                    ui.label(
                        egui::RichText::new(count_badge)
                            .font(FontId::proportional(11.0))
                            .color(Theme::TEXT_SECONDARY),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button(
                                egui::RichText::new("Refresh")
                                    .font(FontId::proportional(11.5))
                                    .color(Theme::ACCENT_BLUE),
                            )
                            .clicked()
                        {
                            self.daemon.refresh_devices();
                        }
                    });
                });

                ui.add_space(8.0);

                // Devices List Card Container
                if self.devices.is_empty() {
                    let empty = egui::Frame::NONE
                        .fill(Theme::SURFACE)
                        .corner_radius(CornerRadius::same(8))
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .inner_margin(Margin::symmetric(16, 20));

                    empty.show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("📱")
                                    .font(FontId::proportional(26.0))
                                    .color(Theme::TEXT_MUTED),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new("No iOS Device Found")
                                    .font(FontId::proportional(13.0))
                                    .strong()
                                    .color(Theme::TEXT_PRIMARY),
                            );
                            ui.add_space(2.0);
                            ui.label(
                                egui::RichText::new("Connect via USB cable and trust this computer on your device.")
                                    .font(FontId::proportional(11.5))
                                    .color(Theme::TEXT_SECONDARY),
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
                                        Theme::ACCENT_BLUE_BG,
                                        Theme::ACCENT_BLUE,
                                        1.5_f32,
                                    )
                                } else {
                                    (
                                        Theme::SURFACE,
                                        Theme::BORDER_SUBTLE,
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
                                        // Left Icon Ring
                                        let icon_box = Rect::from_min_size(ui.cursor().min, Vec2::new(32.0, 32.0));
                                        ui.painter().circle_filled(
                                            icon_box.center(),
                                            16.0_f32,
                                            if is_selected { Theme::SURFACE } else { Theme::SURFACE_MUTED },
                                        );
                                        ui.painter().text(
                                            icon_box.center(),
                                            Align2::CENTER_CENTER,
                                            "📱",
                                            FontId::proportional(16.0),
                                            Color32::BLACK,
                                        );
                                        ui.add_space(38.0);

                                        // Device Info Stack
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                let name_label = egui::RichText::new(&dev.name)
                                                    .font(FontId::proportional(13.5))
                                                    .strong()
                                                    .color(Theme::TEXT_PRIMARY);

                                                if ui.selectable_label(is_selected, name_label).clicked() {
                                                    self.selected_udid = Some(dev.udid.clone());
                                                    self.daemon.select_device(&dev.udid);
                                                }
                                            });

                                            ui.label(
                                                egui::RichText::new(format!("{} · iOS {}", dev.model, dev.ios_version))
                                                    .font(FontId::proportional(11.5))
                                                    .color(Theme::TEXT_SECONDARY),
                                            );

                                            let conn_label = format!("● Connected via {}", dev.connection_type);
                                            ui.label(
                                                egui::RichText::new(conn_label)
                                                    .font(FontId::proportional(10.5))
                                                    .color(Theme::SUCCESS_GREEN),
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

                // Segmented Tab Picker (macOS Pill Style)
                let tab_container = egui::Frame::NONE
                    .fill(Theme::SURFACE_MUTED)
                    .corner_radius(CornerRadius::same(7))
                    .inner_margin(Margin::symmetric(3, 3));

                tab_container.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let btn_width = (ui.available_width() - 8.0) / 2.0;

                        let is_loc = self.active_tab == SidebarTab::Locations;
                        let loc_btn = egui::Button::new(
                            egui::RichText::new("Saved Places")
                                .font(FontId::proportional(12.5))
                                .strong()
                                .color(if is_loc { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED }),
                        )
                        .fill(if is_loc { Theme::SURFACE } else { Color32::TRANSPARENT })
                        .stroke(if is_loc {
                            Stroke::new(1.0_f32, Theme::BORDER_SUBTLE)
                        } else {
                            Stroke::NONE
                        })
                        .corner_radius(CornerRadius::same(5));

                        if ui.add_sized([btn_width, 24.0], loc_btn).clicked() {
                            self.active_tab = SidebarTab::Locations;
                        }

                        let is_routes = self.active_tab == SidebarTab::Routes;
                        let routes_btn = egui::Button::new(
                            egui::RichText::new("GPX Routes")
                                .font(FontId::proportional(12.5))
                                .strong()
                                .color(if is_routes { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED }),
                        )
                        .fill(if is_routes { Theme::SURFACE } else { Color32::TRANSPARENT })
                        .stroke(if is_routes {
                            Stroke::new(1.0_f32, Theme::BORDER_SUBTLE)
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
                                .color(Theme::TEXT_MUTED),
                        );

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .button(
                                    egui::RichText::new("+ Save Current")
                                        .font(FontId::proportional(11.5))
                                        .color(Theme::ACCENT_BLUE),
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
                                    .fill(Theme::SURFACE)
                                    .corner_radius(CornerRadius::same(7))
                                    .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                                    .inner_margin(Margin::symmetric(12, 9));

                                row.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Category color dot
                                        let cat_color = match loc.category.as_str() {
                                            "work" => Color32::from_rgb(59, 130, 246),
                                            "home" => Color32::from_rgb(168, 85, 247),
                                            _ => Color32::from_rgb(100, 116, 139),
                                        };

                                        ui.label(egui::RichText::new("●").color(cat_color).font(FontId::proportional(10.0)));

                                        ui.vertical(|ui| {
                                            if ui
                                                .link(
                                                    egui::RichText::new(&loc.name)
                                                        .font(FontId::proportional(13.0))
                                                        .strong()
                                                        .color(Theme::TEXT_PRIMARY),
                                                )
                                                .clicked()
                                            {
                                                loc_to_jump = Some((loc.latitude, loc.longitude));
                                            }

                                            ui.label(
                                                egui::RichText::new(format!("{:.4}, {:.4}", loc.latitude, loc.longitude))
                                                    .font(FontId::monospace(10.5))
                                                    .color(Theme::TEXT_MUTED),
                                            );
                                        });

                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            if ui.small_button("🗑").on_hover_text("Delete location").clicked() {
                                                loc_to_remove = Some(loc.id.clone());
                                            }
                                            if ui
                                                .button(
                                                    egui::RichText::new("Set")
                                                        .font(FontId::proportional(11.0))
                                                        .color(Theme::ACCENT_BLUE),
                                                )
                                                .on_hover_text("Teleport device directly to this place")
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
                // TAB 2: GPX TRACKS & ROUTES
                else {
                    let import_btn = egui::Button::new(
                        egui::RichText::new("📂 Import GPX File...")
                            .font(FontId::proportional(13.0))
                            .color(Theme::ACCENT_BLUE)
                            .strong(),
                    )
                    .fill(Theme::ACCENT_BLUE_BG)
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(191, 219, 254)))
                    .corner_radius(CornerRadius::same(6));

                    if ui.add_sized([ui.available_width(), 32.0], import_btn).clicked() {
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
                                    .fill(Theme::SURFACE)
                                    .corner_radius(CornerRadius::same(7))
                                    .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                                    .inner_margin(Margin::symmetric(12, 10));

                                card.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new(&route.name)
                                                    .font(FontId::proportional(13.0))
                                                    .strong()
                                                    .color(Theme::TEXT_PRIMARY),
                                            );
                                            ui.label(
                                                egui::RichText::new(format!("{} waypoints", route.waypoints.len()))
                                                    .font(FontId::proportional(11.0))
                                                    .color(Theme::TEXT_MUTED),
                                            );
                                        });

                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            if ui.small_button("🗑").clicked() {
                                                route_to_remove = Some(route.id.clone());
                                            }
                                            if ui
                                                .button(
                                                    egui::RichText::new("▶ Play")
                                                        .font(FontId::proportional(11.5))
                                                        .color(Theme::ACCENT_BLUE),
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
                                    .fill(Theme::SURFACE)
                                    .corner_radius(CornerRadius::same(8))
                                    .stroke(Stroke::new(1.5_f32, Theme::ACCENT_BLUE))
                                    .inner_margin(Margin::symmetric(14, 12));

                                console.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new("ROUTE PLAYBACK")
                                                .font(FontId::proportional(10.5))
                                                .strong()
                                                .color(Theme::TEXT_MUTED),
                                        );
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            let (badge_text, badge_color) = if self.route_state == "running" {
                                                ("● RUNNING", Theme::SUCCESS_GREEN)
                                            } else {
                                                ("❚❚ PAUSED", Theme::WARNING_AMBER)
                                            };
                                            ui.label(
                                                egui::RichText::new(badge_text)
                                                    .font(FontId::proportional(10.5))
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
                                            "Progress: waypoint {} of {}",
                                            self.route_progress_current, self.route_progress_total
                                        ))
                                        .font(FontId::proportional(11.0))
                                        .color(Theme::TEXT_SECONDARY),
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
        // 4. MAIN MAP CANVAS (OpenStreetMap + Precision Glass HUD)
        // =========================================================================
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Theme::APP_BG))
            .show(ctx, |ui| {
                let map = Map::new(Some(&mut self.tiles), &mut self.map_memory, self.pos);
                let response = ui.add(map);

                let painter = ui.painter_at(response.rect);
                let center = response.rect.center();

                // Professional Precision Crosshairs with Target Ring
                let crosshair_color = Color32::from_rgba_unmultiplied(225, 29, 72, 230); // Rose 600
                let crosshair_stroke = Stroke::new(1.5_f32, crosshair_color);

                // Horizontal & Vertical Sight Lines
                painter.line_segment(
                    [
                        Pos2::new(center.x - 14.0, center.y),
                        Pos2::new(center.x - 4.0, center.y),
                    ],
                    crosshair_stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x + 4.0, center.y),
                        Pos2::new(center.x + 14.0, center.y),
                    ],
                    crosshair_stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x, center.y - 14.0),
                        Pos2::new(center.x, center.y - 4.0),
                    ],
                    crosshair_stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(center.x, center.y + 4.0),
                        Pos2::new(center.x, center.y + 14.0),
                    ],
                    crosshair_stroke,
                );

                // Center Sight Circle (ForeFlight / Avionics style)
                painter.circle_stroke(center, 4.0_f32, crosshair_stroke);
                painter.circle_filled(center, 1.5_f32, crosshair_color);

                // Floating Glassmorphic Coordinates Pill (Bottom Left)
                let (lat, lon) = self.current_center_coordinates();
                let hud_pos = Pos2::new(response.rect.left() + 20.0, response.rect.bottom() - 44.0);
                let hud_rect = Rect::from_min_size(hud_pos, Vec2::new(220.0, 30.0));

                painter.rect_filled(
                    hud_rect,
                    CornerRadius::same(7),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 240),
                );
                painter.rect_stroke(
                    hud_rect,
                    CornerRadius::same(7),
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 0, 0, 35)),
                    StrokeKind::Outside,
                );
                painter.text(
                    hud_rect.center(),
                    Align2::CENTER_CENTER,
                    format!("{lat:.6}°, {lon:.6}°"),
                    FontId::monospace(12.0),
                    Theme::TEXT_PRIMARY,
                );

                // Floating Zoom Controls (+ / −) Top Right
                let zoom_top_pos = Pos2::new(response.rect.right() - 44.0, response.rect.top() + 18.0);
                let zoom_in_rect = Rect::from_min_size(zoom_top_pos, Vec2::new(28.0, 28.0));
                let zoom_out_rect = Rect::from_min_size(zoom_top_pos + Vec2::new(0.0, 32.0), Vec2::new(28.0, 28.0));

                painter.rect_filled(zoom_in_rect, CornerRadius::same(5), Theme::SURFACE);
                painter.rect_stroke(zoom_in_rect, CornerRadius::same(5), Stroke::new(1.0_f32, Theme::BORDER_SUBTLE), StrokeKind::Outside);
                painter.text(zoom_in_rect.center(), Align2::CENTER_CENTER, "+", FontId::proportional(16.0), Theme::TEXT_PRIMARY);

                painter.rect_filled(zoom_out_rect, CornerRadius::same(5), Theme::SURFACE);
                painter.rect_stroke(zoom_out_rect, CornerRadius::same(5), Stroke::new(1.0_f32, Theme::BORDER_SUBTLE), StrokeKind::Outside);
                painter.text(zoom_out_rect.center(), Align2::CENTER_CENTER, "−", FontId::proportional(16.0), Theme::TEXT_PRIMARY);

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
        // 5. NATIVE MODAL WINDOWS
        // =========================================================================

        if self.show_password_modal {
            egui::Window::new("Admin Privileges Required")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("iOS 17+ devices require root permissions to initialize the RemoteXPC tunnel daemon.");
                    ui.add_space(8.0);
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.password_input).password(true));

                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui.button("Authenticate").clicked() {
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
                    ui.label("Bookmark Name:");
                    ui.text_edit_singleline(&mut self.save_name_input);

                    ui.add_space(6.0);
                    let (lat, lon) = self.current_center_coordinates();
                    ui.label(
                        egui::RichText::new(format!("Target: {lat:.6}°, {lon:.6}°"))
                            .font(FontId::monospace(11.0))
                            .color(Theme::TEXT_MUTED),
                    );

                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
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
                            .color(Theme::TEXT_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new("Native Rust Edition · v1.0.0")
                            .font(FontId::proportional(12.0))
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.add_space(10.0);
                    ui.label("High-performance iOS Location Simulation tool.");
                    ui.label("• Rendering: Pure Rust GPU (Metal/OpenGL)");
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
