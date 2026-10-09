//! Portable-first application settings.
//!
//! Attempts to keep the settings file next to the executable (truly portable,
//! e.g. on a USB stick). Falls back to the OS per-user config directory when
//! that location is not writable (installed builds, Android, ...).

use crate::errors::Error;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const FILE_NAME: &str = "safe-text.settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Idle minutes before auto-lock. 0 disables auto-lock.
    pub autolock_minutes: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            autolock_minutes: 5,
        }
    }
}

fn portable_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    Some(dir.join(FILE_NAME))
}

fn system_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(FILE_NAME)
}

fn read_from(path: &Path) -> Option<Settings> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub fn load(app: &AppHandle) -> Settings {
    if let Some(p) = portable_path() {
        if let Some(s) = read_from(&p) {
            return s;
        }
    }
    read_from(&system_path(app)).unwrap_or_default()
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), Error> {
    let json = serde_json::to_vec_pretty(settings).map_err(|e| Error::Io(e.to_string()))?;

    if let Some(p) = portable_path() {
        if std::fs::write(&p, &json).is_ok() {
            return Ok(());
        }
    }

    let dir = system_path(app);
    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&dir, &json)?;
    Ok(())
}
