// English messages. Keys are shared with zh-CN.js — keep the two files in sync
// when adding new strings.
export default {
  common: {
    ok: 'OK',
    cancel: 'Cancel',
    save: 'Save',
    retry: 'Retry',
    gotIt: 'Got it',
    confirmTitle: 'Confirm',
    errorTitle: 'Error',
    processing: 'Working…',
    untitled: 'untitled.txt'
  },

  toolbar: {
    new: 'New',
    open: 'Open',
    recent: 'Recent',
    save: 'Save',
    saveAs: 'Save As',
    changePassword: 'Change Password',
    lock: 'Lock',
    settings: 'Settings'
  },

  recent: {
    empty: 'No recent files',
    emptyHintEnabled: 'Files you open will appear here',
    emptyHintDisabled: 'Turn on “Remember recent files” in Settings to start recording',
    clear: 'Clear recent files',
    clearConfirm: 'Clear the recent files list?',
    cleared: 'Recent files cleared'
  },

  status: {
    noDoc: 'No document open',
    notOnDisk: ' (not saved to disk yet)',
    unsaved: 'Unsaved',
    saved: 'Saved'
  },

  editor: {
    placeholderUnlocked: 'Type here; it will be encrypted automatically when you save…',
    placeholderLocked: 'Click “New” to create an encrypted document, or “Open” an existing encrypted .txt file'
  },

  password: {
    label: 'Password',
    newLabel: 'New password',
    confirmLabel: 'Confirm password',
    titleNew: 'Set a password for the new document',
    titleOpen: 'Enter the password to open',
    titleChange: 'Set a new password',
    errEmpty: 'Please enter the password',
    errMismatch: 'The passwords do not match',
    errTooShort: 'The password must be at least 6 characters',
    warnNotSaved: 'The password is not stored — please remember it; if lost, the file content cannot be recovered.'
  },

  dialog: {
    discard: 'There are unsaved changes. Continuing to {action} will discard them. Continue?',
    cannotOpen: 'Cannot open'
  },

  actions: {
    new: 'create',
    open: 'open',
    lock: 'lock'
  },

  settings: {
    title: 'Settings',
    autolock: 'Idle auto-lock (minutes, 0 to disable)',
    rememberRecent: 'Remember recently opened files',
    rememberRecentHint: 'When enabled, file paths are stored only in the local settings file for quick reopening; turning it off and saving clears them immediately.',
    language: 'Language',
    languageAuto: 'Auto (follow system)',
    languageZh: '简体中文',
    languageEn: 'English',
    saved: 'Settings saved'
  },

  notify: {
    created: 'Created and saved',
    opened: 'Opened',
    saved: 'Saved',
    passwordChanged: 'Password changed',
    passwordChangedSaved: 'Password changed and saved',
    locked: 'Locked'
  },

  title: {
    encrypted: 'Encrypted',
    unlocked: 'Unlocked'
  },

  dialogs: {
    encryptedText: 'Encrypted text (*.txt)',
    allFiles: 'All files'
  },

  errors: {
    not_encrypted: 'Not a valid encrypted file',
    bad_password: 'Wrong password or corrupted file',
    no_session: 'No unlocked document',
    not_text: 'Decryption succeeded, but the content is not valid text',
    unsupported_version: 'Unsupported file version: {detail}',
    unsupported_kdf: 'Unsupported key derivation algorithm: {detail}',
    unsupported_cipher: 'Unsupported cipher: {detail}',
    kdf: 'Key derivation failed: {detail}',
    cipher: 'Encryption/decryption failed: {detail}',
    io: 'File operation failed: {detail}',
    unknown: 'Unknown error'
  }
}
