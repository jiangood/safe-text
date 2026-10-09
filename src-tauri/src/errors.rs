use std::fmt;

/// Errors surfaced to the UI. `NotEncrypted` and `BadPassword` map to the
/// two user-facing messages required by the spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Missing/short/magic-mismatched file -> "不是有效加密文件".
    NotEncrypted,
    /// Unsupported container version.
    UnsupportedVersion(u8),
    /// Unsupported key-derivation function id.
    UnsupportedKdf(u8),
    /// Unsupported cipher id.
    UnsupportedCipher(u8),
    /// GCM authentication failed -> "密码错误或文件已损坏".
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotEncrypted => write!(f, "不是有效加密文件"),
            Error::UnsupportedVersion(v) => write!(f, "不支持的文件版本: {v}"),
            Error::UnsupportedKdf(k) => write!(f, "不支持的口令派生算法: {k}"),
            Error::UnsupportedCipher(c) => write!(f, "不支持的加密算法: {c}"),
            Error::BadPassword => write!(f, "密码错误或文件已损坏"),
            Error::Kdf(m) => write!(f, "口令派生失败: {m}"),
            Error::Cipher(m) => write!(f, "加解密失败: {m}"),
            Error::NoSession => write!(f, "没有已解锁的文档"),
            Error::NotText => write!(f, "解密成功，但内容不是有效的文本"),
            Error::Io(m) => write!(f, "文件操作失败: {m}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

/// All Tauri commands return `Result<_, String>` so the message can be
/// shown directly in a Quasar dialog.
impl From<Error> for String {
    fn from(e: Error) -> Self {
        e.to_string()
    }
}
