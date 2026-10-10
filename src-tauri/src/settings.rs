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

/// Maximum number of paths kept in the opt-in recent list.
pub const MAX_RECENT: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Idle minutes before auto-lock. 0 disables auto-lock.
    pub autolock_minutes: u32,
    /// Opt-in: remember recently opened file paths. Off by default so that no
    /// paths are persisted unless the user explicitly asks for it.
    pub remember_recent: bool,
    /// Most-recent-first list of file paths. Only ever populated while
    /// `remember_recent` is true, and cleared the moment it is turned off.
    pub recent_files: Vec<String>,
    /// UI language setting: "auto" (follow the system), "zh", or "en".
    pub language: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            autolock_minutes: 5,
            remember_recent: false,
            recent_files: Vec::new(),
            language: "auto".to_string(),
        }
    }
}

impl Settings {
    /// Move `path` to the front of the recent list (dedup, capped). No-op when
    /// remembering is disabled, so a path is never stored by accident.
    pub fn push_recent(&mut self, path: &str) {
        if !self.remember_recent || path.is_empty() {
            return;
        }
        self.recent_files.retain(|p| p != path);
        self.recent_files.insert(0, path.to_string());
        self.recent_files.truncate(MAX_RECENT);
    }

    /// Drop every remembered path.
    pub fn clear_recent(&mut self) {
        self.recent_files.clear();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_keep_no_paths() {
        let s = Settings::default();
        assert!(!s.remember_recent);
        assert!(s.recent_files.is_empty());
        assert_eq!(s.language, "auto");
    }

    #[test]
    fn missing_language_defaults_to_auto() {
        // Older settings files predate the `language` field.
        let s: Settings = serde_json::from_str(r#"{"autolock_minutes":10}"#).unwrap();
        assert_eq!(s.language, "auto");
        assert_eq!(s.autolock_minutes, 10);
    }

    #[test]
    fn push_is_ignored_until_opted_in() {
        let mut s = Settings::default();
        s.push_recent("C:/a.txt");
        assert!(s.recent_files.is_empty());

        s.remember_recent = true;
        s.push_recent("C:/a.txt");
        assert_eq!(s.recent_files, vec!["C:/a.txt".to_string()]);
    }

    #[test]
    fn push_dedups_and_orders_most_recent_first() {
        let mut s = Settings {
            remember_recent: true,
            ..Settings::default()
        };
        s.push_recent("a");
        s.push_recent("b");
        s.push_recent("a");
        assert_eq!(s.recent_files, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn push_caps_the_list() {
        let mut s = Settings {
            remember_recent: true,
            ..Settings::default()
        };
        for i in 0..(MAX_RECENT + 5) {
            s.push_recent(&format!("file-{i}"));
        }
        assert_eq!(s.recent_files.len(), MAX_RECENT);
        assert_eq!(s.recent_files[0], format!("file-{}", MAX_RECENT + 4));
    }

    #[test]
    fn clear_recent_drops_everything() {
        let mut s = Settings {
            remember_recent: true,
            ..Settings::default()
        };
        s.push_recent("a");
        s.clear_recent();
        assert!(s.recent_files.is_empty());
    }
}
