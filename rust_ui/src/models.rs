use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct iOSDevice {
    pub udid: String,
    pub name: String,
    pub model: String,
    pub ios_version: String,
    pub connection_type: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub icon_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavedLocation {
    pub id: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub created_at: String,
}

fn default_category() -> String {
    "other".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GPXWaypoint {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: Option<f64>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GPXRoute {
    pub id: String,
    pub name: String,
    pub waypoints: Vec<GPXWaypoint>,
    #[serde(default)]
    pub imported_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TunnelStatus {
    pub status: Option<String>,
    #[serde(default)]
    pub running: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestion {
    pub display_name: String,
    pub lat: f64,
    pub lon: f64,
}
