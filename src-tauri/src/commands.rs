//! Tauri command layer. All crypto happens here (in Rust); the frontend is
//! responsible only for file I/O (so Android `content://` URIs work through
//! the official fs plugin) and presentation.

use crate::crypto;
use crate::errors::Error;
use crate::format::KdfParams;
use crate::session::{Session, Unlocked};
use crate::settings::{self, Settings};
use tauri::{AppHandle, State};

/// Encrypt a brand-new document. Unlocks the session with a freshly derived key.
#[tauri::command]
pub fn new_document(
    session: State<'_, Session>,
    password: String,
    text: String,
) -> Result<Vec<u8>, String> {
    let (bytes, salt, key) = crypto::encrypt_new(
        text.as_bytes(),
        password.as_bytes(),
        KdfParams::DEFAULT,
    )?;
    session.with(|s| {
        s.set(Unlocked {
            key,
            salt,
            params: KdfParams::DEFAULT,
        })
    });
    Ok(bytes)
}

/// Decrypt and open an existing document. Unlocks the session with the
/// key/salt/params read from the file header.
#[tauri::command]
pub fn open_document(
    session: State<'_, Session>,
    password: String,
    data: Vec<u8>,
) -> Result<String, String> {
    let (plain, header, key) = crypto::decrypt(&data, password.as_bytes())?;
    let text = String::from_utf8(plain).map_err(|_| Error::NotText)?;
    session.with(|s| {
        s.set(Unlocked {
            key,
            salt: header.salt,
            params: header.params,
        })
    });
    Ok(text)
}

/// Re-encrypt the current document in place (fresh nonce) using the cached key.
#[tauri::command]
pub fn save_document(session: State<'_, Session>, text: String) -> Result<Vec<u8>, String> {
    let unlocked = session.with(|s| s.snapshot()).ok_or(Error::NoSession)?;
    let bytes = crypto::reseal(
        &unlocked.key,
        unlocked.salt,
        unlocked.params,
        text.as_bytes(),
    )?;
    Ok(bytes)
}

/// Change the password: derive a brand-new salt+key and re-encrypt.
#[tauri::command]
pub fn change_password(
    session: State<'_, Session>,
    new_password: String,
    text: String,
) -> Result<Vec<u8>, String> {
    let (bytes, salt, key) = crypto::encrypt_new(
        text.as_bytes(),
        new_password.as_bytes(),
        KdfParams::DEFAULT,
    )?;
    session.with(|s| {
        s.set(Unlocked {
            key,
            salt,
            params: KdfParams::DEFAULT,
        })
    });
    Ok(bytes)
}

/// Forget the cached key (zeroized on drop).
#[tauri::command]
pub fn lock(session: State<'_, Session>) {
    session.with(|s| s.clear());
}

/// Cheap check: does this look like a SafeText container?
#[tauri::command]
pub fn probe_file(data: Vec<u8>) -> bool {
    crypto::probe(&data)
}

/// Read a file by path (used for desktop drag-and-drop, where no dialog
/// scope grant is available). Dialog-mediated reads go through the fs plugin.
#[tauri::command]
pub fn read_file(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| Error::Io(e.to_string()).into())
}

/// Write bytes to a path (used for files opened without a dialog scope, e.g.
/// drag-and-drop or the recent list). Dialog-mediated writes use the fs plugin.
#[tauri::command]
pub fn write_file(path: String, data: Vec<u8>) -> Result<(), String> {
    std::fs::write(&path, &data).map_err(|e| Error::Io(e.to_string()).into())
}

#[tauri::command]
pub fn load_settings(app: AppHandle) -> Result<Settings, String> {
    Ok(settings::load(&app))
}

#[tauri::command]
pub fn save_settings(app: AppHandle, mut settings: Settings) -> Result<(), String> {
    // Never persist paths once the user has turned remembering off.
    if !settings.remember_recent {
        settings.clear_recent();
    }
    settings::save(&app, &settings).map_err(String::from)
}

/// Record `path` in the opt-in recent list (no-op unless the user enabled it).
#[tauri::command]
pub fn add_recent_file(app: AppHandle, path: String) -> Result<Settings, String> {
    let mut settings = settings::load(&app);
    settings.push_recent(&path);
    settings::save(&app, &settings).map_err(String::from)?;
    Ok(settings)
}

/// Forget every remembered file path.
#[tauri::command]
pub fn clear_recent_files(app: AppHandle) -> Result<Settings, String> {
    let mut settings = settings::load(&app);
    settings.clear_recent();
    settings::save(&app, &settings).map_err(String::from)?;
    Ok(settings)
}
