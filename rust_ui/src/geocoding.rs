use crate::models::SearchSuggestion;
use serde::Deserialize;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

#[derive(Deserialize)]
struct NominatimItem {
    display_name: String,
    lat: String,
    lon: String,
}

pub struct GeocodingClient {
    tx: Sender<String>,
    rx: Receiver<Vec<SearchSuggestion>>,
}

impl GeocodingClient {
    pub fn new() -> Self {
        let (tx, task_rx) = channel::<String>();
        let (result_tx, rx) = channel::<Vec<SearchSuggestion>>();

        thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .user_agent("LocationSimulator-Rust/1.0 (contact: github)")
                .build()
                .ok();

            while let Ok(query) = task_rx.recv() {
                let trimmed = query.trim().to_string();
                if trimmed.is_empty() {
                    let _ = result_tx.send(Vec::new());
                    continue;
                }

                // Check if query is already coordinates: "lat, lon"
                if let Some((lat, lon)) = parse_coordinates(&trimmed) {
                    let _ = result_tx.send(vec![SearchSuggestion {
                        display_name: format!("Coordinates: {lat:.6}, {lon:.6}"),
                        lat,
                        lon,
                    }]);
                    continue;
                }

                if let Some(ref client) = client {
                    let url = format!(
                        "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=5",
                        urlencoding::encode(&trimmed)
                    );

                    let suggestions = match client.get(&url).send() {
                        Ok(resp) => {
                            if let Ok(items) = resp.json::<Vec<NominatimItem>>() {
                                items
                                    .into_iter()
                                    .filter_map(|item| {
                                        let lat = item.lat.parse::<f64>().ok()?;
                                        let lon = item.lon.parse::<f64>().ok()?;
                                        Some(SearchSuggestion {
                                            display_name: item.display_name,
                                            lat,
                                            lon,
                                        })
                                    })
                                    .collect()
                            } else {
                                Vec::new()
                            }
                        }
                        Err(_) => Vec::new(),
                    };

                    let _ = result_tx.send(suggestions);
                } else {
                    let _ = result_tx.send(Vec::new());
                }
            }
        });

        Self { tx, rx }
    }

    pub fn search(&self, query: String) {
        let _ = self.tx.send(query);
    }

    pub fn poll_results(&self) -> Option<Vec<SearchSuggestion>> {
        let mut latest = None;
        while let Ok(res) = self.rx.try_recv() {
            latest = Some(res);
        }
        latest
    }
}

pub fn parse_coordinates(text: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = text
        .split([',', ' ', '\t', ';'])
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() == 2 {
        if let (Ok(lat), Ok(lon)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
            if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon) {
                return Some((lat, lon));
            }
        }
    }
    None
}
