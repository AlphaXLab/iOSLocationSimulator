use crate::models::{GPXRoute, SavedLocation, TunnelStatus, iOSDevice};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
pub enum DaemonEvent {
    Ready {
        locations: Vec<SavedLocation>,
        routes: Vec<GPXRoute>,
    },
    DevicesUpdated {
        devices: Vec<iOSDevice>,
        selected_udid: Option<String>,
    },
    SimulationResult {
        success: bool,
        message: String,
    },
    LocationUpdated {
        latitude: f64,
        longitude: f64,
    },
    RouteProgress {
        current: usize,
        total: usize,
    },
    RouteStateChanged {
        state: String,
    },
    SavedLocationsUpdated(Vec<SavedLocation>),
    GpxRoutesUpdated(Vec<GPXRoute>),
    TunnelStatus(TunnelStatus),
    Error(String),
}

pub struct DaemonClient {
    cmd_tx: Sender<String>,
    event_rx: Receiver<DaemonEvent>,
    next_id: AtomicU64,
    _child: Arc<Mutex<Option<Child>>>,
}

impl DaemonClient {
    pub fn spawn() -> Result<Self, String> {
        let (cmd_tx, cmd_rx) = channel::<String>();
        let (event_tx, event_rx) = channel::<DaemonEvent>();

        // Find core_daemon.py in current dir, parent dir, or alongside executable
        let daemon_script = Self::find_daemon_script()?;

        let mut child = Command::new("python3")
            .arg(&daemon_script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn python3 {}: {e}", daemon_script.display()))?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to capture stdin of daemon".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout of daemon".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Failed to capture stderr of daemon".to_string())?;

        // Stderr logger thread
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                eprintln!("[Python Daemon Log] {line}");
            }
        });

        // Stdin writer thread
        thread::spawn(move || {
            while let Ok(cmd) = cmd_rx.recv() {
                if stdin.write_all(cmd.as_bytes()).is_err() || stdin.write_all(b"\n").is_err() {
                    break;
                }
                let _ = stdin.flush();
            }
        });

        // Stdout event / response reader thread
        let event_tx_clone = event_tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().flatten() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
                    if let Some(event_name) = val.get("event").and_then(|v| v.as_str()) {
                        let data = val.get("data").cloned().unwrap_or(Value::Null);
                        Self::handle_event(event_name, data, &event_tx_clone);
                    }
                }
            }
        });

        Ok(Self {
            cmd_tx,
            event_rx,
            next_id: AtomicU64::new(1),
            _child: Arc::new(Mutex::new(Some(child))),
        })
    }

    fn find_daemon_script() -> Result<PathBuf, String> {
        let candidates = [
            PathBuf::from("core_daemon.py"),
            PathBuf::from("../core_daemon.py"),
        ];

        for path in &candidates {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let candidate = parent.join("core_daemon.py");
                if candidate.exists() {
                    return Ok(candidate);
                }
                let candidate2 = parent.join("../core_daemon.py");
                if candidate2.exists() {
                    return Ok(candidate2);
                }
            }
        }

        Ok(PathBuf::from("core_daemon.py"))
    }

    fn handle_event(event: &str, data: Value, tx: &Sender<DaemonEvent>) {
        match event {
            "ready" => {
                let locations = serde_json::from_value(data.get("locations").cloned().unwrap_or(Value::Null))
                    .unwrap_or_default();
                let routes = serde_json::from_value(data.get("routes").cloned().unwrap_or(Value::Null))
                    .unwrap_or_default();
                let _ = tx.send(DaemonEvent::Ready { locations, routes });
            }
            "devices_updated" => {
                let devices = serde_json::from_value(data.get("devices").cloned().unwrap_or(Value::Null))
                    .unwrap_or_default();
                let selected_udid = data
                    .get("selected_udid")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let _ = tx.send(DaemonEvent::DevicesUpdated {
                    devices,
                    selected_udid,
                });
            }
            "simulation_result" => {
                let success = data.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                let message = data
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let _ = tx.send(DaemonEvent::SimulationResult { success, message });
            }
            "location_updated" => {
                if let (Some(lat), Some(lon)) = (
                    data.get("latitude").and_then(|v| v.as_f64()),
                    data.get("longitude").and_then(|v| v.as_f64()),
                ) {
                    let _ = tx.send(DaemonEvent::LocationUpdated {
                        latitude: lat,
                        longitude: lon,
                    });
                }
            }
            "route_progress" => {
                let cur = data.get("current").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let tot = data.get("total").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let _ = tx.send(DaemonEvent::RouteProgress {
                    current: cur,
                    total: tot,
                });
            }
            "route_state_changed" => {
                let state = data
                    .get("state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("idle")
                    .to_string();
                let _ = tx.send(DaemonEvent::RouteStateChanged { state });
            }
            "saved_locations_updated" => {
                if let Ok(locs) = serde_json::from_value::<Vec<SavedLocation>>(data) {
                    let _ = tx.send(DaemonEvent::SavedLocationsUpdated(locs));
                }
            }
            "gpx_routes_updated" => {
                if let Ok(routes) = serde_json::from_value::<Vec<GPXRoute>>(data) {
                    let _ = tx.send(DaemonEvent::GpxRoutesUpdated(routes));
                }
            }
            "tunnel_status" => {
                if let Ok(status) = serde_json::from_value::<TunnelStatus>(data) {
                    let _ = tx.send(DaemonEvent::TunnelStatus(status));
                }
            }
            "device_error" => {
                let err = data
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown device error")
                    .to_string();
                let _ = tx.send(DaemonEvent::Error(err));
            }
            _ => {}
        }
    }

    pub fn poll_events(&self) -> Vec<DaemonEvent> {
        let mut events = Vec::new();
        while let Ok(evt) = self.event_rx.try_recv() {
            events.push(evt);
        }
        events
    }

    pub fn send_command(&self, method: &str, params: Value) {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let req = json!({
            "id": id,
            "method": method,
            "params": params
        });
        let _ = self.cmd_tx.send(req.to_string());
    }

    #[allow(dead_code)]
    pub fn select_device(&self, udid: &str) {
        self.send_command("select_device", json!({ "udid": udid }));
    }

    pub fn refresh_devices(&self) {
        self.send_command("refresh_devices", json!({}));
    }

    pub fn set_location(&self, udid: Option<&str>, lat: f64, lon: f64) {
        let mut params = json!({
            "latitude": lat,
            "longitude": lon
        });
        if let Some(u) = udid {
            params["udid"] = json!(u);
        }
        self.send_command("set_location", params);
    }

    pub fn clear_location(&self, udid: Option<&str>) {
        let mut params = json!({});
        if let Some(u) = udid {
            params["udid"] = json!(u);
        }
        self.send_command("clear_location", params);
    }

    pub fn start_tunnel(&self) {
        self.send_command("start_tunnel", json!({}));
    }

    pub fn submit_tunnel_password(&self, password: &str) {
        self.send_command("submit_tunnel_password", json!({ "password": password }));
    }

    pub fn cancel_tunnel_password(&self) {
        self.send_command("cancel_tunnel_password", json!({}));
    }

    pub fn stop_tunnel(&self) {
        self.send_command("stop_tunnel", json!({}));
    }

    pub fn add_saved_location(&self, name: &str, lat: f64, lon: f64, category: &str) {
        self.send_command(
            "add_saved_location",
            json!({
                "name": name,
                "latitude": lat,
                "longitude": lon,
                "category": category
            }),
        );
    }

    pub fn remove_saved_location(&self, id: &str) {
        self.send_command("remove_saved_location", json!({ "id": id }));
    }

    pub fn import_gpx(&self, file_path: &str) {
        self.send_command("import_gpx", json!({ "file_path": file_path }));
    }

    pub fn remove_gpx_route(&self, id: &str) {
        self.send_command("remove_gpx_route", json!({ "id": id }));
    }

    pub fn start_route(&self, route_id: &str, speed: f64, r#loop: bool) {
        self.send_command(
            "start_route",
            json!({
                "route_id": route_id,
                "speed": speed,
                "loop": r#loop
            }),
        );
    }

    #[allow(dead_code)]
    pub fn pause_route(&self) {
        self.send_command("pause_route", json!({}));
    }

    #[allow(dead_code)]
    pub fn resume_route(&self) {
        self.send_command("resume_route", json!({}));
    }

    #[allow(dead_code)]
    pub fn stop_route(&self) {
        self.send_command("stop_route", json!({}));
    }
}

impl Drop for DaemonClient {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(json!({"id": 0, "method": "quit"}).to_string());
        if let Ok(mut lock) = self._child.lock() {
            if let Some(mut child) = lock.take() {
                let _ = child.kill();
            }
        }
    }
}
