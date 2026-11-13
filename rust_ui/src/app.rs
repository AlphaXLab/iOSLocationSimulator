use crate::config::AppConfig;
use crate::daemon::{DaemonClient, DaemonEvent};
use crate::geocoding::{parse_coordinates, GeocodingClient};
use crate::icons::{self, Icon};
use crate::models::{GPXRoute, SavedLocation, SearchSuggestion, TunnelStatus, iOSDevice};
use crate::theme;
use crate::widgets;
use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, Layout, Margin, Pos2, Rect, Stroke,
    StrokeKind, Vec2,
};
use walkers::{lat_lon, HttpOptions, HttpTiles, Map, MapMemory, Position, TileId};

#[derive(PartialEq, Clone, Copy)]
enum SidebarTab {
    Locations,
    Routes,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MapStyle {
    Google,
    Hybrid,
    Osm,
    Dark,
}

// ---------------------------------------------------------------------------
// Tile Sources
// ---------------------------------------------------------------------------

/// Google Maps Tile Source (Roadmap "m", Satellite Hybrid "y", Terrain "p")
#[derive(Clone, Default)]
pub struct GoogleMapsSource {
    pub api_key: String,
    pub layer: &'static str,
}

impl walkers::sources::TileSource for GoogleMapsSource {
    fn tile_url(&self, tile_id: TileId) -> String {
        let servers = ['0', '1', '2', '3'];
        let s = servers[(tile_id.x + tile_id.y) as usize % servers.len()];
        if self.api_key.is_empty() {
            format!(
                "https://mt{}.google.com/vt/lyrs={}&x={}&y={}&z={}",
                s, self.layer, tile_id.x, tile_id.y, tile_id.zoom
            )
        } else {
            format!(
                "https://mt{}.google.com/vt/lyrs={}&x={}&y={}&z={}&key={}",
                s, self.layer, tile_id.x, tile_id.y, tile_id.zoom, self.api_key
            )
        }
    }

    fn attribution(&self) -> walkers::sources::Attribution {
        walkers::sources::Attribution {
            text: "© Google Maps",
            url: "https://maps.google.com",
            logo_light: None,
            logo_dark: None,
        }
    }
}

/// ArcGIS Dark Gray Canvas (100% free, NO watermark, NO API key required)
#[derive(Clone, Copy, Default)]
pub struct ArcGisDark;

impl walkers::sources::TileSource for ArcGisDark {
    fn tile_url(&self, tile_id: TileId) -> String {
        format!(
            "https://services.arcgisonline.com/ArcGIS/rest/services/Canvas/World_Dark_Gray_Base/MapServer/tile/{}/{}/{}",
            tile_id.zoom, tile_id.y, tile_id.x
        )
    }

    fn attribution(&self) -> walkers::sources::Attribution {
        walkers::sources::Attribution {
            text: "© Esri, HERE, Garmin, © OpenStreetMap contributors",
            url: "https://www.arcgis.com",
            logo_light: None,
            logo_dark: None,
        }
    }
}

/// OpenStreetMap Standard Tiles (100% free and open, zero API key required)
#[derive(Clone, Copy, Default)]
pub struct OpenStreetMapSource;

impl walkers::sources::TileSource for OpenStreetMapSource {
    fn tile_url(&self, tile_id: TileId) -> String {
        format!(
            "https://tile.openstreetmap.org/{}/{}/{}.png",
            tile_id.zoom, tile_id.x, tile_id.y
        )
    }

    fn attribution(&self) -> walkers::sources::Attribution {
        walkers::sources::Attribution {
            text: "© OpenStreetMap contributors",
            url: "https://www.openstreetmap.org/copyright",
            logo_light: None,
            logo_dark: None,
        }
    }
}

pub struct LocationSimulatorApp {
    daemon: DaemonClient,
    geocoding: GeocodingClient,

    // Map
    tiles: HttpTiles,
    map_memory: MapMemory,
    pos: Position,
    map_style: MapStyle,
    egui_ctx: egui::Context,

    // App Data
    devices: Vec<iOSDevice>,
    selected_udid: Option<String>,
    saved_locations: Vec<SavedLocation>,
    gpx_routes: Vec<GPXRoute>,
    tunnel_status: TunnelStatus,

    // Navigation & Layout
    active_tab: SidebarTab,

    // Search (Google Maps style floating pill)
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

    // Modals & Config
    config: AppConfig,
    show_api_key_modal: bool,
    api_key_input: String,
    show_password_modal: bool,
    password_input: String,
    show_save_modal: bool,
    save_name_input: String,
    save_category_input: String,
    show_about_modal: bool,
}

impl LocationSimulatorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        theme::apply(&cc.egui_ctx, true);

        let daemon = DaemonClient::spawn().expect("Failed to initialize Python daemon sidecar");
        let geocoding = GeocodingClient::new();
        let config = AppConfig::load();
        let show_api_key_modal = config.google_maps_api_key.trim().is_empty();
        let api_key_input = config.google_maps_api_key.clone();

        let http_options = HttpOptions {
            cache: dirs_cache_dir(),
            ..Default::default()
        };
        // Default to Google Maps Roadmap
        let tiles = HttpTiles::with_options(
            GoogleMapsSource {
                api_key: config.google_maps_api_key.clone(),
                layer: "m",
            },
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
            map_style: MapStyle::Google,
            egui_ctx: cc.egui_ctx.clone(),
            config,
            show_api_key_modal,
            api_key_input,
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

    fn set_map_style(&mut self, style: MapStyle) {
        if self.map_style == style {
            return;
        }
        self.map_style = style;
        let http_options = HttpOptions {
            cache: dirs_cache_dir(),
            ..Default::default()
        };
        self.tiles = match style {
            MapStyle::Google => {
                HttpTiles::with_options(
                    GoogleMapsSource {
                        api_key: self.config.google_maps_api_key.clone(),
                        layer: "m",
                    },
                    http_options,
                    self.egui_ctx.clone(),
                )
            }
            MapStyle::Hybrid => {
                HttpTiles::with_options(
                    GoogleMapsSource {
                        api_key: self.config.google_maps_api_key.clone(),
                        layer: "y",
                    },
                    http_options,
                    self.egui_ctx.clone(),
                )
            }
            MapStyle::Osm => {
                HttpTiles::with_options(OpenStreetMapSource, http_options, self.egui_ctx.clone())
            }
            MapStyle::Dark => {
                HttpTiles::with_options(ArcGisDark, http_options, self.egui_ctx.clone())
            }
        };
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
                    self.status_message = "Connected to iOS Daemon".to_string();
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
                DaemonEvent::TunnelStatus(status) => {
                    if let Some(err) = &status.error {
                        if err.contains("password") || err.contains("Authentication") || err.contains("sudo") {
                            self.show_password_modal = true;
                        }
                    }
                    self.tunnel_status = status;
                }
                DaemonEvent::SavedLocationsUpdated(locations) => {
                    self.saved_locations = locations;
                }
                DaemonEvent::GpxRoutesUpdated(routes) => {
                    self.gpx_routes = routes;
                }
                DaemonEvent::Error(message) => {
                    self.status_message = format!("Error: {message}");
                    self.status_is_error = true;
                }
            }
        }

        // Poll geocoding async responses
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

fn truncate_chars(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

impl eframe::App for LocationSimulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_background_events();
        let pal = theme::palette(true);

        // =========================================================================
        // 1. SPACIOUS TABLEX STATUS FOOTER BAR (Generous 56pt height - comfortable & open)
        // =========================================================================
        egui::TopBottomPanel::bottom("tablex_footer")
            .exact_height(56.0)
            .frame(
                egui::Frame::NONE
                    .fill(pal.chrome)
                    .inner_margin(Margin::symmetric(20, 10))
                    .stroke(Stroke::new(1.0_f32, pal.separator)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // Status glowing dot
                    let dot_color = if self.status_is_error {
                        pal.danger
                    } else if self.status_message.starts_with("Setting") || self.status_message.starts_with("Starting") {
                        pal.accent
                    } else {
                        pal.ok
                    };

                    let dot_center = ui.cursor().min + Vec2::new(6.0, 18.0);
                    ui.painter().circle_filled(dot_center, 4.5_f32, dot_color);
                    ui.add_space(20.0);

                    // Status Message + Realtime Map Coordinates
                    let (cur_lat, cur_lon) = self.current_center_coordinates();
                    ui.label(
                        egui::RichText::new(&self.status_message)
                            .font(theme::ui(13.0))
                            .color(if self.status_is_error { pal.danger } else { pal.text }),
                    );

                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(format!("({cur_lat:.6}°, {cur_lon:.6}°)",))
                            .font(theme::mono(11.5))
                            .color(pal.text_faint),
                    );

                    // Right Side Action Buttons: Generous, comfortable 36pt height buttons with breathing room
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let has_device = self.selected_udid.is_some() || !self.devices.is_empty();

                        // Primary Action: "Simulate Location" (TableX Accent Blue #4C8DFF)
                        let set_btn = egui::Button::new(
                            egui::RichText::new("Simulate Location")
                                .font(theme::semi(13.0))
                                .color(Color32::WHITE),
                        )
                        .fill(if has_device {
                            pal.accent
                        } else {
                            pal.button
                        })
                        .stroke(Stroke::new(1.0_f32, if has_device { pal.accent } else { pal.separator }))
                        .corner_radius(CornerRadius::same(theme::RADIUS_SM));

                        if ui
                            .add_sized([150.0, 36.0], set_btn)
                            .on_hover_text("Send simulated GPS coordinates to selected iOS device")
                            .clicked()
                        {
                            self.daemon.set_location(self.selected_udid.as_deref(), cur_lat, cur_lon);
                            self.status_message = format!("Simulating position: {cur_lat:.5}, {cur_lon:.5}...");
                            self.status_is_error = false;
                        }

                        ui.add_space(10.0);

                        // Clear Location (TableX Danger Pill)
                        let clear_btn = egui::Button::new(
                            egui::RichText::new("Clear Location")
                                .font(theme::ui(12.5))
                                .color(pal.danger),
                        )
                        .fill(pal.danger_bg)
                        .stroke(Stroke::new(1.0_f32, pal.separator))
                        .corner_radius(CornerRadius::same(theme::RADIUS_SM));

                        if ui
                            .add_sized([120.0, 36.0], clear_btn)
                            .on_hover_text("Stop location simulation on iOS device and restore real GPS")
                            .clicked()
                        {
                            self.daemon.clear_location(self.selected_udid.as_deref());
                            self.status_message = "Restored real GPS location".to_string();
                            self.status_is_error = false;
                        }

                        ui.add_space(10.0);

                        // Save Bookmark button
                        let save_btn = egui::Button::new(
                            egui::RichText::new("Save Bookmark")
                                .font(theme::ui(12.5))
                                .color(pal.text),
                        )
                        .fill(pal.elevated)
                        .stroke(Stroke::new(1.0_f32, pal.separator))
                        .corner_radius(CornerRadius::same(theme::RADIUS_SM));

                        if ui.add_sized([125.0, 36.0], save_btn).clicked() {
                            let (lat, lon) = self.current_center_coordinates();
                            self.save_name_input = format!("Location {:.4}, {:.4}", lat, lon);
                            self.show_save_modal = true;
                        }
                    });
                });
            });

        // =========================================================================
        // 2. UNIFIED LEFT NAVBAR (Contains Device Pill, Tunnel, Import, Bookmarks)
        // =========================================================================
        // NO MORE TOP BAR! The left sidebar reaches the very top of the window,
        // cleanly accommodating macOS traffic lights and all primary controls.
        egui::SidePanel::left("unified_left_navbar")
            .exact_width(300.0)
            .resizable(false)
            .frame(
                egui::Frame::NONE
                    .fill(pal.sidebar)
                    .inner_margin(Margin {
                        left: 14,
                        right: 14,
                        top: if cfg!(target_os = "macos") { 42 } else { 14 },
                        bottom: 12,
                    })
                    .stroke(Stroke::new(1.0_f32, pal.separator)),
            )
            .show(ctx, |ui| {
                // Header Row: App Brand Title & Utility Buttons
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Location Simulator")
                            .font(theme::semi(13.5))
                            .color(pal.text),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // About Dialog Button
                        if widgets::toolbar_icon(ui, Icon::More, "About Location Simulator", false).clicked() {
                            self.show_about_modal = true;
                        }

                        // Google Maps API Key Setup
                        if widgets::toolbar_icon(ui, Icon::Sliders, "Google Maps API Key Setup", false).clicked() {
                            self.api_key_input = self.config.google_maps_api_key.clone();
                            self.show_api_key_modal = true;
                        }

                        // Import GPX Route
                        if widgets::toolbar_icon(ui, Icon::Import, "Import GPX Track", false).clicked() {
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

                        // Refresh Connected Devices
                        if widgets::toolbar_icon(ui, Icon::Refresh, "Refresh Connected Devices", false).clicked() {
                            self.daemon.refresh_devices();
                        }
                    });
                });

                ui.add_space(10.0);

                // --- CONNECTED DEVICE CARD ---
                let (dev_name, dev_desc, dev_dot) = if let Some(selected) = self
                    .devices
                    .iter()
                    .find(|d| Some(&d.udid) == self.selected_udid.as_ref())
                {
                    (
                        selected.name.clone(),
                        format!("{} · iOS {}", selected.model, selected.ios_version),
                        pal.ok,
                    )
                } else if let Some(first) = self.devices.first() {
                    (
                        first.name.clone(),
                        format!("{} · iOS {}", first.model, first.ios_version),
                        pal.ok,
                    )
                } else {
                    (
                        "No Device Connected".to_string(),
                        "Connect iPhone/iPad via USB".to_string(),
                        pal.text_faint,
                    )
                };

                let card_rect = ui
                    .allocate_exact_size(egui::vec2(ui.available_width(), 48.0), egui::Sense::hover())
                    .0;

                ui.painter().rect_filled(card_rect, CornerRadius::same(theme::RADIUS_SM), pal.chrome);
                ui.painter().rect_stroke(
                    card_rect,
                    CornerRadius::same(theme::RADIUS_SM),
                    Stroke::new(1.0_f32, pal.separator),
                    StrokeKind::Inside,
                );

                let plug_icon_rect = Rect::from_center_size(
                    card_rect.left_center() + Vec2::new(18.0, 0.0),
                    Vec2::new(16.0, 16.0),
                );
                icons::paint(ui.painter(), plug_icon_rect, Icon::Plug, dev_dot);

                ui.painter().text(
                    card_rect.left_center() + Vec2::new(36.0, -8.0),
                    Align2::LEFT_CENTER,
                    &dev_name,
                    theme::semi(12.5),
                    pal.text,
                );
                ui.painter().text(
                    card_rect.left_center() + Vec2::new(36.0, 8.0),
                    Align2::LEFT_CENTER,
                    &dev_desc,
                    theme::mono(10.0),
                    pal.text_muted,
                );

                ui.add_space(6.0);

                // --- TUNNEL STATUS BAR (iOS 17+) ---
                let tunnel_running = self.tunnel_status.running;
                let tunnel_rect = ui
                    .allocate_exact_size(egui::vec2(ui.available_width(), 32.0), egui::Sense::click());

                let tunnel_bg = if tunnel_rect.1.hovered() { pal.hover } else { pal.chrome };
                ui.painter().rect_filled(tunnel_rect.0, CornerRadius::same(theme::RADIUS_SM), tunnel_bg);
                ui.painter().rect_stroke(
                    tunnel_rect.0,
                    CornerRadius::same(theme::RADIUS_SM),
                    Stroke::new(1.0_f32, if tunnel_running { pal.accent } else { pal.separator }),
                    StrokeKind::Inside,
                );

                let tunnel_icon_rect = Rect::from_center_size(
                    tunnel_rect.0.left_center() + Vec2::new(18.0, 0.0),
                    Vec2::new(14.0, 14.0),
                );
                icons::paint(
                    ui.painter(),
                    tunnel_icon_rect,
                    if tunnel_running { Icon::Lock } else { Icon::Unlock },
                    if tunnel_running { pal.ok } else { pal.text_muted },
                );

                ui.painter().text(
                    tunnel_rect.0.left_center() + Vec2::new(36.0, 0.0),
                    Align2::LEFT_CENTER,
                    if tunnel_running { "RemoteXPC Tunnel: Active" } else { "RemoteXPC Tunnel: Inactive" },
                    theme::ui(11.5),
                    if tunnel_running { pal.ok } else { pal.text_muted },
                );

                // Start/Stop action text on right
                ui.painter().text(
                    tunnel_rect.0.right_center() - Vec2::new(12.0, 0.0),
                    Align2::RIGHT_CENTER,
                    if tunnel_running { "Stop" } else { "Start" },
                    theme::semi(11.0),
                    if tunnel_running { pal.danger } else { pal.accent },
                );

                if tunnel_rect.1.clicked() {
                    if tunnel_running {
                        self.daemon.stop_tunnel();
                    } else {
                        self.daemon.start_tunnel();
                        self.status_message = "Starting RemoteXPC tunnel daemon...".to_string();
                    }
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                // Navigation Tabs (Saved Places / GPX Routes)
                ui.horizontal(|ui| {
                    let w = (ui.available_width() - 4.0) / 2.0;

                    let loc_active = self.active_tab == SidebarTab::Locations;
                    let loc_btn = egui::Button::new(
                        egui::RichText::new("Saved Places")
                            .font(theme::semi(12.0))
                            .color(if loc_active { pal.text } else { pal.text_muted }),
                    )
                    .fill(if loc_active { pal.selection } else { pal.button })
                    .stroke(Stroke::new(1.0_f32, if loc_active { pal.accent } else { pal.separator }))
                    .corner_radius(CornerRadius::same(theme::RADIUS_SM));

                    if ui.add_sized([w, 28.0], loc_btn).clicked() {
                        self.active_tab = SidebarTab::Locations;
                    }

                    let r_active = self.active_tab == SidebarTab::Routes;
                    let r_btn = egui::Button::new(
                        egui::RichText::new("GPX Routes")
                            .font(theme::semi(12.0))
                            .color(if r_active { pal.text } else { pal.text_muted }),
                    )
                    .fill(if r_active { pal.selection } else { pal.button })
                    .stroke(Stroke::new(1.0_f32, if r_active { pal.accent } else { pal.separator }))
                    .corner_radius(CornerRadius::same(theme::RADIUS_SM));

                    if ui.add_sized([w, 28.0], r_btn).clicked() {
                        self.active_tab = SidebarTab::Routes;
                    }
                });

                ui.add_space(8.0);

                // TAB 1: SAVED PLACES
                if self.active_tab == SidebarTab::Locations {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("BOOKMARKS ({})", self.saved_locations.len()))
                                .font(theme::semi(10.5))
                                .color(pal.text_faint),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button(egui::RichText::new("+ Save").font(theme::semi(11.0))).clicked() {
                                let (lat, lon) = self.current_center_coordinates();
                                self.save_name_input = format!("Location {:.4}, {:.4}", lat, lon);
                                self.show_save_modal = true;
                            }
                        });
                    });

                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .id_salt("saved_locations_scroll")
                        .show(ui, |ui| {
                            let mut loc_to_jump = None;
                            let mut loc_to_set = None;
                            let mut loc_to_remove = None;

                            for loc in &self.saved_locations {
                                let item_h = 44.0_f32;
                                let (item_rect, item_resp) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), item_h),
                                    egui::Sense::click(),
                                );

                                let bg = if item_resp.hovered() { pal.hover } else { pal.chrome };
                                ui.painter().rect_filled(item_rect, CornerRadius::same(theme::RADIUS_SM), bg);
                                ui.painter().rect_stroke(
                                    item_rect,
                                    CornerRadius::same(theme::RADIUS_SM),
                                    Stroke::new(1.0_f32, pal.separator),
                                    StrokeKind::Inside,
                                );

                                // Category dot
                                let cat_color = match loc.category.as_str() {
                                    "work" => pal.accent,
                                    "home" => pal.ok,
                                    _ => pal.text_muted,
                                };
                                ui.painter().circle_filled(item_rect.left_center() + Vec2::new(10.0, 0.0), 3.5_f32, cat_color);

                                // Place title
                                ui.painter().text(
                                    item_rect.left_center() + Vec2::new(22.0, -7.0),
                                    Align2::LEFT_CENTER,
                                    &loc.name,
                                    theme::semi(12.0),
                                    pal.text,
                                );

                                // Coordinates
                                ui.painter().text(
                                    item_rect.left_center() + Vec2::new(22.0, 8.0),
                                    Align2::LEFT_CENTER,
                                    format!("{:.4}, {:.4}", loc.latitude, loc.longitude),
                                    theme::mono(10.0),
                                    pal.text_faint,
                                );

                                // Right Action Buttons: Generous, comfortable 28x28 hit targets!
                                let right_center = item_rect.right_center();
                                let del_rect = Rect::from_center_size(right_center - Vec2::new(16.0, 0.0), Vec2::new(28.0, 28.0));
                                let set_rect = Rect::from_center_size(right_center - Vec2::new(48.0, 0.0), Vec2::new(28.0, 28.0));

                                let is_del_hovered = item_resp.interact_pointer_pos().map_or(false, |p| del_rect.contains(p));
                                let is_set_hovered = item_resp.interact_pointer_pos().map_or(false, |p| set_rect.contains(p));

                                // Set button
                                let set_bg = if is_set_hovered { pal.selection } else { Color32::TRANSPARENT };
                                ui.painter().rect_filled(set_rect, CornerRadius::same(theme::RADIUS_SM), set_bg);
                                if is_set_hovered {
                                    ui.painter().rect_stroke(set_rect, CornerRadius::same(theme::RADIUS_SM), Stroke::new(1.0_f32, pal.accent), StrokeKind::Inside);
                                }
                                let set_icon_rect = Rect::from_center_size(set_rect.center(), Vec2::new(15.0, 15.0));
                                icons::paint(ui.painter(), set_icon_rect, Icon::Play, pal.accent);

                                // Delete button: Generous 28x28 target with unmistakable danger hover feedback
                                let del_bg = if is_del_hovered { pal.danger_bg } else { Color32::TRANSPARENT };
                                ui.painter().rect_filled(del_rect, CornerRadius::same(theme::RADIUS_SM), del_bg);
                                if is_del_hovered {
                                    ui.painter().rect_stroke(del_rect, CornerRadius::same(theme::RADIUS_SM), Stroke::new(1.0_f32, pal.danger), StrokeKind::Inside);
                                }
                                let del_icon_rect = Rect::from_center_size(del_rect.center(), Vec2::new(15.0, 15.0));
                                icons::paint(
                                    ui.painter(),
                                    del_icon_rect,
                                    Icon::Trash,
                                    if is_del_hovered { pal.danger } else { pal.text_muted },
                                );

                                if item_resp.clicked() {
                                    if let Some(pos) = item_resp.interact_pointer_pos() {
                                        if del_rect.contains(pos) {
                                            loc_to_remove = Some(loc.id.clone());
                                        } else if set_rect.contains(pos) {
                                            loc_to_set = Some((loc.latitude, loc.longitude));
                                        } else {
                                            loc_to_jump = Some((loc.latitude, loc.longitude));
                                        }
                                    }
                                }

                                ui.add_space(3.0);
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
                // TAB 2: GPX ROUTES
                else {
                    if ui.add_sized([ui.available_width(), 28.0], egui::Button::new("+ Import GPX...").fill(pal.button)).clicked() {
                        if let Some(path) = rfd::FileDialog::new().add_filter("GPX Routes", &["gpx"]).pick_file() {
                            if let Some(path_str) = path.to_str() {
                                self.daemon.import_gpx(path_str);
                            }
                        }
                    }

                    ui.add_space(6.0);

                    egui::ScrollArea::vertical()
                        .id_salt("routes_scroll")
                        .show(ui, |ui| {
                            let mut route_to_remove = None;
                            let mut route_to_start = None;

                            for route in &self.gpx_routes {
                                let item_h = 44.0_f32;
                                let (item_rect, item_resp) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), item_h),
                                    egui::Sense::click(),
                                );

                                let bg = if item_resp.hovered() { pal.hover } else { pal.chrome };
                                ui.painter().rect_filled(item_rect, CornerRadius::same(theme::RADIUS_SM), bg);
                                ui.painter().rect_stroke(
                                    item_rect,
                                    CornerRadius::same(theme::RADIUS_SM),
                                    Stroke::new(1.0_f32, pal.separator),
                                    StrokeKind::Inside,
                                );

                                // Route Title
                                ui.painter().text(
                                    item_rect.left_center() + Vec2::new(12.0, -7.0),
                                    Align2::LEFT_CENTER,
                                    &route.name,
                                    theme::semi(12.0),
                                    pal.text,
                                );

                                // Waypoints
                                ui.painter().text(
                                    item_rect.left_center() + Vec2::new(12.0, 8.0),
                                    Align2::LEFT_CENTER,
                                    format!("{} waypoints", route.waypoints.len()),
                                    theme::mono(10.0),
                                    pal.text_faint,
                                );

                                let right_center = item_rect.right_center();
                                let del_rect = Rect::from_center_size(right_center - Vec2::new(16.0, 0.0), Vec2::new(28.0, 28.0));
                                let play_rect = Rect::from_center_size(right_center - Vec2::new(48.0, 0.0), Vec2::new(28.0, 28.0));

                                let is_del_hovered = item_resp.interact_pointer_pos().map_or(false, |p| del_rect.contains(p));
                                let is_play_hovered = item_resp.interact_pointer_pos().map_or(false, |p| play_rect.contains(p));

                                // Play route button
                                let play_bg = if is_play_hovered { pal.selection } else { Color32::TRANSPARENT };
                                ui.painter().rect_filled(play_rect, CornerRadius::same(theme::RADIUS_SM), play_bg);
                                if is_play_hovered {
                                    ui.painter().rect_stroke(play_rect, CornerRadius::same(theme::RADIUS_SM), Stroke::new(1.0_f32, pal.accent), StrokeKind::Inside);
                                }
                                let play_icon_rect = Rect::from_center_size(play_rect.center(), Vec2::new(15.0, 15.0));
                                icons::paint(ui.painter(), play_icon_rect, Icon::Play, pal.accent);

                                // Delete route button (Generous 28x28 target)
                                let del_bg = if is_del_hovered { pal.danger_bg } else { Color32::TRANSPARENT };
                                ui.painter().rect_filled(del_rect, CornerRadius::same(theme::RADIUS_SM), del_bg);
                                if is_del_hovered {
                                    ui.painter().rect_stroke(del_rect, CornerRadius::same(theme::RADIUS_SM), Stroke::new(1.0_f32, pal.danger), StrokeKind::Inside);
                                }
                                let del_icon_rect = Rect::from_center_size(del_rect.center(), Vec2::new(15.0, 15.0));
                                icons::paint(
                                    ui.painter(),
                                    del_icon_rect,
                                    Icon::Trash,
                                    if is_del_hovered { pal.danger } else { pal.text_muted },
                                );

                                if item_resp.clicked() {
                                    if let Some(pos) = item_resp.interact_pointer_pos() {
                                        if del_rect.contains(pos) {
                                            route_to_remove = Some(route.id.clone());
                                        } else if play_rect.contains(pos) {
                                            route_to_start = Some(route.id.clone());
                                        }
                                    }
                                }

                                ui.add_space(3.0);
                            }

                            if let Some(id) = route_to_remove {
                                self.daemon.remove_gpx_route(&id);
                            }
                            if let Some(id) = route_to_start {
                                self.daemon.start_route(&id, self.route_speed, self.route_loop);
                                self.status_message = "Starting route simulation...".to_string();
                            }

                            // Route simulation active console
                            if self.route_state != "idle" {
                                ui.add_space(6.0);
                                let console_rect = ui.allocate_exact_size(egui::vec2(ui.available_width(), 64.0), egui::Sense::hover()).0;
                                ui.painter().rect_filled(console_rect, CornerRadius::same(theme::RADIUS_SM), pal.elevated);
                                ui.painter().rect_stroke(console_rect, CornerRadius::same(theme::RADIUS_SM), Stroke::new(1.0_f32, pal.accent), StrokeKind::Inside);

                                ui.painter().text(
                                    console_rect.left_top() + Vec2::new(8.0, 10.0),
                                    Align2::LEFT_TOP,
                                    format!("SIMULATION · {}", self.route_state.to_uppercase()),
                                    theme::semi(10.0),
                                    pal.accent,
                                );

                                let progress = if self.route_progress_total > 0 {
                                    self.route_progress_current as f32 / self.route_progress_total as f32
                                } else {
                                    0.0
                                };

                                let bar_rect = Rect::from_min_size(console_rect.left_top() + Vec2::new(8.0, 26.0), Vec2::new(console_rect.width() - 16.0, 6.0));
                                ui.painter().rect_filled(bar_rect, CornerRadius::ZERO, pal.input);
                                let fill_bar = Rect::from_min_size(bar_rect.min, Vec2::new(bar_rect.width() * progress, bar_rect.height()));
                                ui.painter().rect_filled(fill_bar, CornerRadius::ZERO, pal.accent);

                                ui.painter().text(
                                    console_rect.left_bottom() + Vec2::new(8.0, -10.0),
                                    Align2::LEFT_BOTTOM,
                                    format!("{}/{} waypoints", self.route_progress_current, self.route_progress_total),
                                    theme::mono(10.0),
                                    pal.text_muted,
                                );
                            }
                        });
                }
            });

        // =========================================================================
        // 3. MAIN MAP CANVAS (Reaches Window Top, With Google Maps Floating Search)
        // =========================================================================
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(pal.canvas))
            .show(ctx, |ui| {
                let map = Map::new(Some(&mut self.tiles), &mut self.map_memory, self.pos);
                let response = ui.add(map);

                let painter = ui.painter_at(response.rect);
                let center = response.rect.center();

                // High-visibility Precision Reticle (Dual-tone for clarity on both light and dark maps)
                let halo_stroke = Stroke::new(2.5_f32, Color32::from_black_alpha(180));
                painter.line_segment([Pos2::new(center.x - 14.0, center.y), Pos2::new(center.x - 4.0, center.y)], halo_stroke);
                painter.line_segment([Pos2::new(center.x + 4.0, center.y), Pos2::new(center.x + 14.0, center.y)], halo_stroke);
                painter.line_segment([Pos2::new(center.x, center.y - 14.0), Pos2::new(center.x, center.y - 4.0)], halo_stroke);
                painter.line_segment([Pos2::new(center.x, center.y + 4.0), Pos2::new(center.x, center.y + 14.0)], halo_stroke);
                painter.circle_stroke(center, 4.0_f32, halo_stroke);

                let reticle_color = pal.accent;
                let reticle_stroke = Stroke::new(1.5_f32, reticle_color);
                painter.line_segment([Pos2::new(center.x - 14.0, center.y), Pos2::new(center.x - 4.0, center.y)], reticle_stroke);
                painter.line_segment([Pos2::new(center.x + 4.0, center.y), Pos2::new(center.x + 14.0, center.y)], reticle_stroke);
                painter.line_segment([Pos2::new(center.x, center.y - 14.0), Pos2::new(center.x, center.y - 4.0)], reticle_stroke);
                painter.line_segment([Pos2::new(center.x, center.y + 4.0), Pos2::new(center.x, center.y + 14.0)], reticle_stroke);
                painter.circle_stroke(center, 4.0_f32, reticle_stroke);
                painter.circle_filled(center, 1.5_f32, reticle_color);

                // =========================================================================
                // FLOATING MAP STYLE SWITCHER (Google / Hybrid / OSM / Dark) Top-Right
                // =========================================================================
                let style_pos = Pos2::new(response.rect.right() - 270.0, response.rect.top() + 20.0);
                let style_rect = Rect::from_min_size(style_pos, Vec2::new(252.0, 36.0));

                painter.rect_filled(
                    style_rect.translate(Vec2::new(0.0, 2.0)),
                    CornerRadius::same(theme::RADIUS_MD),
                    Color32::from_black_alpha(50),
                );
                painter.rect_filled(style_rect, CornerRadius::same(theme::RADIUS_MD), pal.elevated);
                painter.rect_stroke(style_rect, CornerRadius::same(theme::RADIUS_MD), Stroke::new(1.0_f32, pal.separator), StrokeKind::Inside);

                let mut style_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(style_rect.shrink(3.0))
                        .layout(Layout::left_to_right(Align::Center)),
                );
                style_ui.spacing_mut().item_spacing.x = 2.0;
                style_ui.spacing_mut().interact_size = egui::vec2(20.0, 28.0);
                style_ui.spacing_mut().button_padding = egui::vec2(6.0, 2.0);

                let styles = [
                    (MapStyle::Google, "Google"),
                    (MapStyle::Hybrid, "Hybrid"),
                    (MapStyle::Osm, "OSM"),
                    (MapStyle::Dark, "Dark"),
                ];
                for (style, label) in styles {
                    let is_active = self.map_style == style;
                    let btn = egui::Button::new(
                        egui::RichText::new(label)
                            .font(theme::semi(11.5))
                            .color(if is_active { pal.accent } else { pal.text_muted }),
                    )
                    .fill(if is_active { pal.selection } else { Color32::TRANSPARENT })
                    .stroke(Stroke::NONE)
                    .corner_radius(CornerRadius::same(theme::RADIUS_SM));

                    if style_ui.add_sized([58.0, 28.0], btn).clicked() {
                        self.set_map_style(style);
                    }
                }

                // =========================================================================
                // FLOATING ZOOM CONTROLS (+ / −) Bottom-Right (Google Maps Style)
                // =========================================================================
                let zoom_bottom_pos = Pos2::new(response.rect.right() - 50.0, response.rect.bottom() - 110.0);
                let zoom_in_rect = Rect::from_min_size(zoom_bottom_pos, Vec2::new(34.0, 34.0));
                let zoom_out_rect = Rect::from_min_size(zoom_bottom_pos + Vec2::new(0.0, 40.0), Vec2::new(34.0, 34.0));

                painter.rect_filled(zoom_in_rect.translate(Vec2::new(0.0, 2.0)), CornerRadius::same(theme::RADIUS_MD), Color32::from_black_alpha(50));
                painter.rect_filled(zoom_in_rect, CornerRadius::same(theme::RADIUS_MD), pal.elevated);
                painter.rect_stroke(zoom_in_rect, CornerRadius::same(theme::RADIUS_MD), Stroke::new(1.0_f32, pal.separator), StrokeKind::Inside);
                painter.text(zoom_in_rect.center(), Align2::CENTER_CENTER, "+", theme::semi(18.0), pal.text);

                painter.rect_filled(zoom_out_rect.translate(Vec2::new(0.0, 2.0)), CornerRadius::same(theme::RADIUS_MD), Color32::from_black_alpha(50));
                painter.rect_filled(zoom_out_rect, CornerRadius::same(theme::RADIUS_MD), pal.elevated);
                painter.rect_stroke(zoom_out_rect, CornerRadius::same(theme::RADIUS_MD), Stroke::new(1.0_f32, pal.separator), StrokeKind::Inside);
                painter.text(zoom_out_rect.center(), Align2::CENTER_CENTER, "−", theme::semi(18.0), pal.text);

                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        if zoom_in_rect.contains(pos) {
                            let _ = self.map_memory.zoom_in();
                        } else if zoom_out_rect.contains(pos) {
                            let _ = self.map_memory.zoom_out();
                        }
                    }
                }

                // =========================================================================
                // INTERACTIVE PINNING: Double-Click or Right-Click anywhere on Map
                // =========================================================================
                // Double-click to instantly center & pin the location
                if response.double_clicked() {
                    if let Some(click_pos) = response.interact_pointer_pos() {
                        let is_on_controls = zoom_in_rect.contains(click_pos)
                            || zoom_out_rect.contains(click_pos)
                            || style_rect.contains(click_pos);
                        if !is_on_controls {
                            let projector = walkers::Projector::new(response.rect, &self.map_memory, self.pos);
                            let geo_pos = projector.unproject(click_pos.to_vec2());
                            self.jump_to(geo_pos.y(), geo_pos.x());
                            self.status_message = format!("Pinned at {:.6}°, {:.6}° (Click 'Simulate Location' to inject)", geo_pos.y(), geo_pos.x());
                        }
                    }
                }
                // Right-click (secondary click) to quick-pin AND simulate immediately
                if response.secondary_clicked() {
                    if let Some(click_pos) = response.interact_pointer_pos() {
                        let is_on_controls = zoom_in_rect.contains(click_pos)
                            || zoom_out_rect.contains(click_pos)
                            || style_rect.contains(click_pos);
                        if !is_on_controls {
                            let projector = walkers::Projector::new(response.rect, &self.map_memory, self.pos);
                            let geo_pos = projector.unproject(click_pos.to_vec2());
                            self.jump_to(geo_pos.y(), geo_pos.x());
                            self.daemon.set_location(self.selected_udid.as_deref(), geo_pos.y(), geo_pos.x());
                            self.status_message = format!("Simulating at {:.6}°, {:.6}°", geo_pos.y(), geo_pos.x());
                            self.status_is_error = false;
                        }
                    }
                }

                // =========================================================================
                // GOOGLE MAPS STYLE FLOATING SEARCH BAR (Top-Left of Map Canvas)
                // =========================================================================
                let search_w = 420.0_f32;
                let search_h = 48.0_f32;
                let search_pos = Pos2::new(response.rect.left() + 20.0, response.rect.top() + 20.0);
                let search_rect = Rect::from_min_size(search_pos, Vec2::new(search_w, search_h));

                // Floating Google Maps Style Pill: Soft Elevation Drop Shadow
                painter.rect_filled(
                    search_rect.translate(Vec2::new(0.0, 3.0)),
                    CornerRadius::same(24),
                    Color32::from_black_alpha(65),
                );
                // Pill Body
                painter.rect_filled(
                    search_rect,
                    CornerRadius::same(24),
                    pal.elevated,
                );
                painter.rect_stroke(
                    search_rect,
                    CornerRadius::same(24),
                    Stroke::new(1.0_f32, if self.search_query.is_empty() { pal.separator } else { pal.accent }),
                    StrokeKind::Inside,
                );

                // Search Icon on Left (vertically and horizontally centered in pill's left curve)
                let search_icon_rect = Rect::from_center_size(search_rect.left_center() + Vec2::new(24.0, 0.0), Vec2::new(18.0, 18.0));
                icons::paint(
                    &painter,
                    search_icon_rect,
                    Icon::Search,
                    if self.search_query.is_empty() { pal.text_muted } else { pal.accent },
                );

                // Text input placed inside the floating search bar (generous height & padding)
                let text_input_rect = Rect::from_min_max(
                    search_rect.left_top() + Vec2::new(46.0, 4.0),
                    search_rect.right_bottom() - Vec2::new(if self.search_query.is_empty() { 20.0 } else { 46.0 }, 4.0),
                );

                let mut text_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(text_input_rect)
                        .layout(Layout::left_to_right(Align::Center)),
                );

                let edit_resp = text_ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Search Google Maps or coordinates...")
                        .font(theme::ui(13.5))
                        .text_color(pal.text)
                        .margin(Margin::symmetric(4, 8))
                        .frame(false)
                        .desired_width(text_input_rect.width()),
                );

                // Trigger geocoding on typing
                if edit_resp.changed() {
                    let query = self.search_query.trim().to_string();
                    if query.len() >= 2 && query != self.last_searched_query {
                        self.last_searched_query = query.clone();
                        self.geocoding.search(query, self.config.google_maps_api_key.clone());
                    } else if query.is_empty() {
                        self.suggestions.clear();
                        self.show_suggestions = false;
                    }
                }

                // Press Enter to jump
                if edit_resp.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Some((lat, lon)) = parse_coordinates(&self.search_query) {
                        self.jump_to(lat, lon);
                        self.show_suggestions = false;
                    } else if let Some(first) = self.suggestions.first() {
                        self.jump_to(first.lat, first.lon);
                        self.show_suggestions = false;
                    }
                }

                // Clear button ('X') when query is not empty
                if !self.search_query.is_empty() {
                    let clear_rect = Rect::from_center_size(search_rect.right_center() - Vec2::new(24.0, 0.0), Vec2::new(26.0, 26.0));
                    let clear_icon_rect = Rect::from_center_size(clear_rect.center(), Vec2::new(14.0, 14.0));
                    icons::paint(&painter, clear_icon_rect, Icon::Close, pal.text_muted);

                    if ui.input(|i| i.pointer.primary_clicked()) {
                        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                            if clear_rect.contains(pos) {
                                self.search_query.clear();
                                self.last_searched_query.clear();
                                self.suggestions.clear();
                                self.show_suggestions = false;
                            }
                        }
                    }
                }

                // Autocomplete Dropdown under Floating Search Bar
                if self.show_suggestions && !self.suggestions.is_empty() {
                    let dropdown_top = search_rect.bottom() + 8.0;
                    let item_h = 40.0_f32;
                    let num_items = self.suggestions.len().min(5);
                    let dropdown_h = num_items as f32 * item_h + 8.0;
                    let dropdown_rect = Rect::from_min_size(Pos2::new(search_rect.left(), dropdown_top), Vec2::new(search_w, dropdown_h));

                    // Drop shadow for dropdown
                    painter.rect_filled(
                        dropdown_rect.translate(Vec2::new(0.0, 4.0)),
                        CornerRadius::same(theme::RADIUS_MD),
                        Color32::from_black_alpha(70),
                    );
                    painter.rect_filled(dropdown_rect, CornerRadius::same(theme::RADIUS_MD), pal.elevated);
                    painter.rect_stroke(dropdown_rect, CornerRadius::same(theme::RADIUS_MD), Stroke::new(1.0_f32, pal.separator), StrokeKind::Inside);

                    let mut dropdown_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(dropdown_rect.shrink(4.0))
                            .layout(Layout::top_down(Align::Min)),
                    );

                    let mut chosen = None;
                    for item in self.suggestions.iter().take(5) {
                        let (row_rect, row_resp) = dropdown_ui.allocate_exact_size(Vec2::new(dropdown_rect.width() - 8.0, item_h), egui::Sense::click());
                        if row_resp.hovered() {
                            dropdown_ui.painter().rect_filled(row_rect, CornerRadius::same(theme::RADIUS_SM), pal.hover);
                        }

                        let item_icon_rect = Rect::from_center_size(row_rect.left_center() + Vec2::new(16.0, 0.0), Vec2::new(14.0, 14.0));
                        icons::paint(dropdown_ui.painter(), item_icon_rect, Icon::Search, pal.accent);

                        let text = truncate_chars(&item.display_name, 50);
                        dropdown_ui.painter().text(
                            row_rect.left_center() + Vec2::new(34.0, 0.0),
                            Align2::LEFT_CENTER,
                            &text,
                            theme::ui(12.5),
                            pal.text,
                        );

                        if row_resp.clicked() {
                            chosen = Some((item.lat, item.lon, item.display_name.clone()));
                        }
                    }

                    if let Some((lat, lon, name)) = chosen {
                        self.jump_to(lat, lon);
                        self.search_query = name;
                        self.show_suggestions = false;
                    }
                }
            });

        // =========================================================================
        // 4. TABLEX MODAL DIALOGS (With Spacious 38pt Textboxes & Clean Actions)
        // =========================================================================

        if self.show_password_modal {
            egui::Window::new("Admin Privileges Required")
                .collapsible(false)
                .resizable(false)
                .order(egui::Order::Foreground)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_width(440.0);
                    ui.label(
                        egui::RichText::new("iOS 17+ devices require root permissions to initialize the RemoteXPC tunnel daemon.")
                            .font(theme::ui(12.5))
                            .color(pal.text_muted),
                    );
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Password:").font(theme::semi(12.5)).color(pal.text));
                    ui.add(widgets::password_input(&mut self.password_input).desired_width(410.0));

                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui.add_sized([140.0, 36.0], egui::Button::new(egui::RichText::new("Authenticate").font(theme::semi(13.0))).fill(pal.accent)).clicked() {
                            self.daemon.submit_tunnel_password(&self.password_input);
                            self.password_input.clear();
                            self.show_password_modal = false;
                        }
                        if ui.add_sized([100.0, 36.0], egui::Button::new(egui::RichText::new("Cancel").font(theme::ui(12.5)))).clicked() {
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
                .order(egui::Order::Foreground)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_width(440.0);
                    ui.label(egui::RichText::new("Bookmark Name:").font(theme::semi(12.5)).color(pal.text));
                    ui.add(widgets::text_input(&mut self.save_name_input, "e.g. Apple Park, Home...").desired_width(410.0));

                    ui.add_space(8.0);
                    let (lat, lon) = self.current_center_coordinates();
                    ui.label(
                        egui::RichText::new(format!("Coordinates: {lat:.6}°, {lon:.6}°"))
                            .font(theme::mono(12.0))
                            .color(pal.accent),
                    );

                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui.add_sized([120.0, 36.0], egui::Button::new(egui::RichText::new("Save").font(theme::semi(13.0))).fill(pal.accent)).clicked() {
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
                        if ui.add_sized([100.0, 36.0], egui::Button::new(egui::RichText::new("Cancel").font(theme::ui(12.5)))).clicked() {
                            self.show_save_modal = false;
                        }
                    });
                });
        }

        if self.show_about_modal {
            egui::Window::new("About Location Simulator")
                .collapsible(false)
                .resizable(false)
                .order(egui::Order::Foreground)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_width(440.0);
                    ui.label(
                        egui::RichText::new("Location Simulator")
                            .font(theme::semi(15.0))
                            .color(pal.text),
                    );
                    ui.label(
                        egui::RichText::new("TableX Design System · Native Rust")
                            .font(theme::ui(11.5))
                            .color(pal.accent),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("High-performance iOS Location Simulation tool.\n• UI: Pure Rust Native GPU (TableX Dark Theme, Lucide Icons)\n• Basemaps: Google Maps, ArcGIS Dark Gray, OpenStreetMap\n• Backend: pymobiledevice3 daemon (iOS 12 – iOS 18+)")
                            .font(theme::ui(12.0))
                            .color(pal.text_muted)
                    );
                    ui.add_space(14.0);
                    if ui.add_sized([100.0, 36.0], egui::Button::new(egui::RichText::new("Close").font(theme::ui(12.5)))).clicked() {
                        self.show_about_modal = false;
                    }
                });
        }

        if self.show_api_key_modal {
            egui::Window::new("Google Maps Platform Setup")
                .collapsible(false)
                .resizable(false)
                .order(egui::Order::Foreground)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_width(460.0);
                    ui.label(
                        egui::RichText::new("Enter your Google Maps Platform API key to enable official Google Maps tiles and worldwide place search in all languages (including UTF-8 & Myanmar).")
                            .font(theme::ui(12.5))
                            .color(pal.text_muted),
                    );
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Google Maps API Key:").font(theme::semi(12.5)).color(pal.text));
                    ui.add(
                        widgets::mono_input(&mut self.api_key_input, "AIzaSy...")
                            .desired_width(430.0),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("Need a key? Visit console.cloud.google.com → APIs & Services")
                            .font(theme::ui(11.0))
                            .color(pal.text_faint),
                    );
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui.add_sized([200.0, 36.0], egui::Button::new(egui::RichText::new("Save & Enable Google Maps").font(theme::semi(13.0))).fill(pal.accent)).clicked() {
                            self.config.google_maps_api_key = self.api_key_input.trim().to_string();
                            self.config.save();
                            self.set_map_style(MapStyle::Google);
                            self.status_message = "Google Maps enabled".to_string();
                            self.show_api_key_modal = false;
                        }
                        if ui.add_sized([160.0, 36.0], egui::Button::new(egui::RichText::new("Continue Without Key").font(theme::ui(12.5)))).clicked() {
                            self.show_api_key_modal = false;
                        }
                    });
                });
        }
    }
}
