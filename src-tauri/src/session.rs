//! In-memory unlocked-document state.
//!
//! The derived key never leaves Rust; the frontend only ever receives
//! ciphertext or plaintext bytes. Locking (or app exit) drops the key,
//! which is zeroized.

use crate::format::{KdfParams, KEY_LEN, SALT_LEN};
use std::sync::Mutex;
use zeroize::Zeroizing;

/// Cached material needed to re-encrypt on save without re-deriving.
pub struct Unlocked {
    pub key: Zeroizing<[u8; KEY_LEN]>,
    pub salt: [u8; SALT_LEN],
    pub params: KdfParams,
}

impl Clone for Unlocked {
    fn clone(&self) -> Self {
        Unlocked {
            key: self.key.clone(),
            salt: self.salt,
            params: self.params,
        }
    }
}

#[derive(Default)]
pub struct Inner {
    state: Option<Unlocked>,
}

impl Inner {
    pub fn set(&mut self, unlocked: Unlocked) {
        self.state = Some(unlocked);
    }

    pub fn clear(&mut self) {
        self.state = None;
    }

    pub fn snapshot(&self) -> Option<Unlocked> {
        self.state.clone()
    }
}

/// Tauri-managed state wrapper.
#[derive(Default)]
pub struct Session(pub Mutex<Inner>);

impl Session {
    pub fn with<R>(&self, f: impl FnOnce(&mut Inner) -> R) -> R {
        let mut guard = self.0.lock().expect("session mutex poisoned");
        f(&mut guard)
    }
}
