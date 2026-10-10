// Thin wrapper around the Rust command layer (crypto) and the official
// Tauri plugins (dialogs + file I/O). Keeping file I/O in the frontend is
// what makes Android `content://` URIs work with the same code path.

import { invoke } from '@tauri-apps/api/core'
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog'
import { readFile as fsReadFile, writeFile as fsWriteFile } from '@tauri-apps/plugin-fs'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { t } from '@/i18n'

/** Normalise whatever Tauri returns (number[] or ArrayBuffer) to Uint8Array. */
function toBytes (value) {
  if (value instanceof Uint8Array) return value
  if (value instanceof ArrayBuffer) return new Uint8Array(value)
  return Uint8Array.from(value)
}

/** Dialog file filters, localized at call time (labels change with the locale). */
function txtFilters () {
  return [
    { name: t('dialogs.encryptedText'), extensions: ['txt'] },
    { name: t('dialogs.allFiles'), extensions: ['*'] }
  ]
}

/**
 * Normalise an error raised by the backend into `{ code, detail }`.
 *
 * Rust commands reject with a JSON string `{"code":"...","message":"..."}`
 * (see `errors.rs`); Tauri plugins (dialog / fs) reject with plain strings.
 * Anything that does not parse becomes `{ code: 'unknown', detail: raw }`.
 */
export function parseError (e) {
  const raw = typeof e === 'string' ? e : (e?.message ?? String(e))
  try {
    const obj = JSON.parse(raw)
    if (obj && typeof obj.code === 'string') {
      return { code: obj.code, detail: obj.message ?? '' }
    }
  } catch {
    // not JSON – fall through to the raw string
  }
  return { code: 'unknown', detail: raw }
}

export const backend = {
  // ---- crypto (Rust) -----------------------------------------------------
  newDocument: (password, text) =>
    invoke('new_document', { password, text }).then(toBytes),

  openDocument: (password, bytes) =>
    invoke('open_document', { password, data: Array.from(bytes) }),

  saveDocument: (text) =>
    invoke('save_document', { text }).then(toBytes),

  changePassword: (newPassword, text) =>
    invoke('change_password', { newPassword, text }).then(toBytes),

  lock: () => invoke('lock'),

  probeFile: (bytes) =>
    invoke('probe_file', { data: Array.from(bytes) }),

  readFileByPath: (path) =>
    invoke('read_file', { path }).then(toBytes),

  writeFileByPath: (path, bytes) =>
    invoke('write_file', { path, data: Array.from(bytes) }),

  // ---- settings ----------------------------------------------------------
  loadSettings: () => invoke('load_settings'),
  saveSettings: (settings) => invoke('save_settings', { settings }),

  // ---- recent files (opt-in) ---------------------------------------------
  addRecentFile: (path) => invoke('add_recent_file', { path }),
  clearRecentFiles: () => invoke('clear_recent_files'),

  // ---- dialogs -----------------------------------------------------------
  pickOpenPath: () =>
    openDialog({ multiple: false, directory: false, filters: txtFilters() }),

  pickSavePath: (defaultName) =>
    saveDialog({ defaultPath: defaultName, filters: txtFilters() }),

  // ---- file I/O (fs plugin; dialog-scoped) -------------------------------
  readFile: (path) => fsReadFile(path),
  writeFile: (path, bytes) => fsWriteFile(path, bytes),

  // ---- drag & drop -------------------------------------------------------
  onDragDrop: async (handler) => {
    const webview = getCurrentWebview()
    return webview.onDragDropEvent(handler)
  }
}
