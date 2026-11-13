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

#[derive(Deserialize)]
struct GoogleGeocodeResponse {
    results: Option<Vec<GoogleGeocodeResult>>,
    status: Option<String>,
}

#[derive(Deserialize)]
struct GoogleGeocodeResult {
    formatted_address: String,
    geometry: GoogleGeometry,
}

#[derive(Deserialize)]
struct GoogleGeometry {
    location: GoogleLocation,
}

#[derive(Deserialize)]
struct GoogleLocation {
    lat: f64,
    lng: f64,
}

#[derive(Deserialize)]
struct PhotonResponse {
    features: Option<Vec<PhotonFeature>>,
}

#[derive(Deserialize)]
struct PhotonFeature {
    geometry: PhotonGeometry,
    properties: PhotonProperties,
}

#[derive(Deserialize)]
struct PhotonGeometry {
    coordinates: Vec<f64>,
}

#[derive(Deserialize)]
struct PhotonProperties {
    name: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
}

pub struct GeocodingClient {
    tx: Sender<(String, String)>,
    rx: Receiver<Vec<SearchSuggestion>>,
}

impl GeocodingClient {
    pub fn new() -> Self {
        let (tx, task_rx) = channel::<(String, String)>();
        let (result_tx, rx) = channel::<Vec<SearchSuggestion>>();

        thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .user_agent("LocationSimulator/2.0 (iOS Location Simulation; contact: github)")
                .build()
                .ok();

            while let Ok((query, api_key)) = task_rx.recv() {
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

                let mut suggestions = Vec::new();

                if let Some(ref client) = client {
                    // 1. Try Google Maps Geocoding API if key provided
                    if !api_key.is_empty() {
                        let g_url = format!(
                            "https://maps.googleapis.com/maps/api/geocode/json?address={}&key={}",
                            urlencoding::encode(&trimmed),
                            api_key
                        );
                        if let Ok(resp) = client.get(&g_url).send() {
                            if let Ok(g_data) = resp.json::<GoogleGeocodeResponse>() {
                                if g_data.status.as_deref() == Some("OK") {
                                    if let Some(results) = g_data.results {
                                        suggestions = results
                                            .into_iter()
                                            .take(5)
                                            .map(|r| SearchSuggestion {
                                                display_name: r.formatted_address,
                                                lat: r.geometry.location.lat,
                                                lon: r.geometry.location.lng,
                                            })
                                            .collect();
                                    }
                                }
                            }
                        }
                    }

                    // 2. If no Google results, use Photon (OpenStreetMap UTF-8 autocomplete)
                    if suggestions.is_empty() {
                        let photon_url = format!(
                            "https://photon.komoot.io/api/?q={}&limit=5",
                            urlencoding::encode(&trimmed)
                        );
                        if let Ok(resp) = client.get(&photon_url).send() {
                            if let Ok(p_data) = resp.json::<PhotonResponse>() {
                                if let Some(features) = p_data.features {
                                    suggestions = features
                                        .into_iter()
                                        .filter_map(|f| {
                                            if f.geometry.coordinates.len() >= 2 {
                                                let lon = f.geometry.coordinates[0];
                                                let lat = f.geometry.coordinates[1];
                                                let mut labels = Vec::new();
                                                if let Some(n) = f.properties.name {
                                                    labels.push(n);
                                                }
                                                if let Some(c) = f.properties.city {
                                                    if !labels.contains(&c) {
                                                        labels.push(c);
                                                    }
                                                }
                                                if let Some(s) = f.properties.state {
                                                    if !labels.contains(&s) {
                                                        labels.push(s);
                                                    }
                                                }
                                                if let Some(co) = f.properties.country {
                                                    labels.push(co);
                                                }
                                                let name = if labels.is_empty() {
                                                    format!("{lat:.5}, {lon:.5}")
                                                } else {
                                                    labels.join(", ")
                                                };
                                                Some(SearchSuggestion {
                                                    display_name: name,
                                                    lat,
                                                    lon,
                                                })
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();
                                }
                            }
                        }
                    }

                    // 3. Fallback to Nominatim if still empty
                    if suggestions.is_empty() {
                        let nom_url = format!(
                            "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=5",
                            urlencoding::encode(&trimmed)
                        );
                        if let Ok(resp) = client.get(&nom_url).send() {
                            if let Ok(items) = resp.json::<Vec<NominatimItem>>() {
                                suggestions = items
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
                                    .collect();
                            }
                        }
                    }
                }

                let _ = result_tx.send(suggestions);
            }
        });

        Self { tx, rx }
    }

    pub fn search(&self, query: String, api_key: String) {
        let _ = self.tx.send((query, api_key));
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
