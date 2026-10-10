use std::fmt;

use serde::Serialize;

/// Errors surfaced to the UI. Each variant carries a stable `code`; the
/// frontend maps it to a localized message (`errors.<code>` in `src/i18n`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Missing/short/magic-mismatched file.
    NotEncrypted,
    /// Unsupported container version.
    UnsupportedVersion(u8),
    /// Unsupported key-derivation function id.
    UnsupportedKdf(u8),
    /// Unsupported cipher id.
    UnsupportedCipher(u8),
    /// GCM authentication failed.
    BadPassword,
    /// Key derivation failed.
    Kdf(String),
    /// Symmetric cipher failed.
    Cipher(String),
    /// No document is currently open/unlocked.
    NoSession,
    /// The decrypted bytes are not valid UTF-8 text.
    NotText,
    /// Filesystem error.
    Io(String),
}

impl Error {
    /// Stable machine-readable code. Keep in sync with `errors.*` in the
    /// frontend locale files.
    pub fn code(&self) -> &'static str {
        match self {
            Error::NotEncrypted => "not_encrypted",
            Error::UnsupportedVersion(_) => "unsupported_version",
            Error::UnsupportedKdf(_) => "unsupported_kdf",
            Error::UnsupportedCipher(_) => "unsupported_cipher",
            Error::BadPassword => "bad_password",
            Error::Kdf(_) => "kdf",
            Error::Cipher(_) => "cipher",
            Error::NoSession => "no_session",
            Error::NotText => "not_text",
            Error::Io(_) => "io",
        }
    }

    /// Optional technical detail (a numeric id or an underlying message),
    /// appended to the localized text by the frontend.
    fn detail(&self) -> Option<String> {
        match self {
            Error::UnsupportedVersion(v) => Some(v.to_string()),
            Error::UnsupportedKdf(k) => Some(k.to_string()),
            Error::UnsupportedCipher(c) => Some(c.to_string()),
            Error::Kdf(m) | Error::Cipher(m) | Error::Io(m) => Some(m.clone()),
            _ => None,
        }
    }
}

/// JSON payload handed to the frontend: `{"code":"...","message":"..."}`.
#[derive(Serialize)]
struct Payload<'a> {
    code: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

/// Human-readable (English) description, used only for logging.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotEncrypted => write!(f, "not a valid encrypted file"),
            Error::UnsupportedVersion(v) => write!(f, "unsupported file version: {v}"),
            Error::UnsupportedKdf(k) => write!(f, "unsupported key derivation: {k}"),
            Error::UnsupportedCipher(c) => write!(f, "unsupported cipher: {c}"),
            Error::BadPassword => write!(f, "wrong password or corrupted file"),
            Error::Kdf(m) => write!(f, "key derivation failed: {m}"),
            Error::Cipher(m) => write!(f, "cipher operation failed: {m}"),
            Error::NoSession => write!(f, "no unlocked document"),
            Error::NotText => write!(f, "decrypted content is not valid text"),
            Error::Io(m) => write!(f, "file operation failed: {m}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

/// All Tauri commands return `Result<_, String>`; the string is a small JSON
/// object carrying a stable `code` so the UI can localize the message.
impl From<Error> for String {
    fn from(e: Error) -> Self {
        let payload = Payload {
            code: e.code(),
            message: e.detail(),
        };
        serde_json::to_string(&payload)
            .unwrap_or_else(|_| format!("{{\"code\":\"{}\"}}", e.code()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable() {
        assert_eq!(Error::NotEncrypted.code(), "not_encrypted");
        assert_eq!(Error::BadPassword.code(), "bad_password");
        assert_eq!(Error::NoSession.code(), "no_session");
        assert_eq!(Error::NotText.code(), "not_text");
        assert_eq!(Error::Io("x".into()).code(), "io");
    }

    #[test]
    fn serializes_to_json_with_code_and_detail() {
        let plain: String = Error::BadPassword.into();
        assert_eq!(plain, r#"{"code":"bad_password"}"#);

        let io: String = Error::Io("Access is denied".into()).into();
        assert_eq!(io, r#"{"code":"io","message":"Access is denied"}"#);

        let ver: String = Error::UnsupportedVersion(2).into();
        assert_eq!(ver, r#"{"code":"unsupported_version","message":"2"}"#);
    }
}
