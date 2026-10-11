use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::widgets::Widget;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placed {
    pub widget: Widget,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    #[serde(default)]
    pub widgets: Vec<Placed>,
    #[serde(default = "behind")]
    pub behind: bool,
    #[serde(default = "city")]
    pub city: String,
}

fn behind() -> bool {
    true
}

fn city() -> String {
    "Toronto".to_string()
}

impl Default for Layout {
    fn default() -> Self {
        Layout {
            widgets: Vec::new(),
            behind: behind(),
            city: city(),
        }
    }
}

fn path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("Dextra").join("layout.json"))
}

pub fn load() -> Layout {
    path()
        .and_then(|file| std::fs::read_to_string(file).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save(layout: &Layout) {
    let Some(file) = path() else {
        return;
    };
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string_pretty(layout) {
        let _ = std::fs::write(file, text);
    }
}
