use bevy::prelude::*;
use bevy::window::WindowPosition;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ── Persisted struct ─────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct WindowGeometry {
    /// Physical screen x of the window's top-left corner.
    pub x: i32,
    /// Physical screen y of the window's top-left corner.
    pub y: i32,
    /// Logical width in pixels.
    pub width: u32,
    /// Logical height in pixels.
    pub height: u32,
}

// ── Disk I/O ─────────────────────────────────────────────────────────────────

fn geometry_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("jaspersworld_test2").join("window.json"))
}

/// Reads saved window geometry from disk. Returns None if the file does not
/// exist or cannot be parsed — callers treat None as "use defaults".
pub fn load_window_geometry() -> Option<WindowGeometry> {
    let path = geometry_path()?;
    let text = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&text).ok()
}

fn save_window_geometry(geom: &WindowGeometry) {
    let Some(path) = geometry_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(geom) {
        let _ = fs::write(&path, json);
    }
}

// ── Bevy system ──────────────────────────────────────────────────────────────

// (x, y, width_bits, height_bits) — bit-cast to avoid f32 equality issues
type GeomSnapshot = (i32, i32, u32, u32);

/// Compares the current window position and size against the last written
/// snapshot. Writes to disk only when something has changed.
/// Runs every Update frame; disk I/O fires only on actual changes.
pub fn persist_window_geometry(
    windows: Query<&Window>,
    mut last: Local<Option<GeomSnapshot>>,
    quit: Res<crate::states::QuitRequested>,
) {
    if quit.0 { return; }
    let Ok(window) = windows.single() else { return };

    // Position only becomes WindowPosition::At once the OS has placed the window.
    let WindowPosition::At(pos) = window.position else { return };

    let w = window.width() as u32;
    let h = window.height() as u32;
    let current: GeomSnapshot = (pos.x, pos.y, w, h);

    if Some(current) == *last {
        return;
    }

    *last = Some(current);

    save_window_geometry(&WindowGeometry {
        x: pos.x,
        y: pos.y,
        width: w,
        height: h,
    });
}
