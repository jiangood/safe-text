//! SafeText on-disk file format (version 1).
//!
//! All multi-byte integers are big-endian.
//!
//! ```text
//! offset  size  field
//! 0       4     magic          b"STXT"
//! 4       1     version        1
//! 5       1     kdf_id         1 = Argon2id
//! 6       1     cipher_id      1 = AES-256-GCM
//! 7       1     reserved       0
//! 8       4     argon2 m_cost  (KiB)
//! 12      4     argon2 t_cost
//! 16      4     argon2 p_cost
//! 20      16    salt
//! 36      12    nonce
//! 48      ...   ciphertext + 16-byte GCM tag
//! ```
//!
//! The 48-byte header is authenticated as AES-GCM associated data (AAD),
//! so any tampering with version/costs/salt/nonce is detected on decrypt.

use crate::errors::Error;

pub const MAGIC: [u8; 4] = *b"STXT";
pub const VERSION: u8 = 1;
pub const KDF_ARGON2ID: u8 = 1;
pub const CIPHER_AES256GCM: u8 = 1;

pub const HEADER_LEN: usize = 48;
pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const TAG_LEN: usize = 16;
pub const KEY_LEN: usize = 32;

/// Argon2id cost parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KdfParams {
    /// Memory cost in KiB.
    pub m_cost: u32,
    /// Time cost (iterations).
    pub t_cost: u32,
    /// Parallelism (lanes).
    pub p_cost: u32,
}

impl KdfParams {
    /// 64 MiB, 3 iterations, 1 lane (OWASP-recommended minimum for Argon2id).
    pub const DEFAULT: KdfParams = KdfParams {
        m_cost: 64 * 1024,
        t_cost: 3,
        p_cost: 1,
    };
}

/// Parsed fixed-size header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub version: u8,
    pub kdf_id: u8,
    pub cipher_id: u8,
    pub params: KdfParams,
    pub salt: [u8; SALT_LEN],
    pub nonce: [u8; NONCE_LEN],
}

impl Header {
    /// Serialize the fixed 48-byte header.
    pub fn encode(&self) -> [u8; HEADER_LEN] {
        let mut b = [0u8; HEADER_LEN];
        b[0..4].copy_from_slice(&MAGIC);
        b[4] = self.version;
        b[5] = self.kdf_id;
        b[6] = self.cipher_id;
        b[7] = 0; // reserved
        b[8..12].copy_from_slice(&self.params.m_cost.to_be_bytes());
        b[12..16].copy_from_slice(&self.params.t_cost.to_be_bytes());
        b[16..20].copy_from_slice(&self.params.p_cost.to_be_bytes());
        b[20..36].copy_from_slice(&self.salt);
        b[36..48].copy_from_slice(&self.nonce);
        b
    }

    /// Parse and validate a header from the start of `data`.
    pub fn decode(data: &[u8]) -> Result<Header, Error> {
        if data.len() < HEADER_LEN {
            return Err(Error::NotEncrypted);
        }
        if data[0..4] != MAGIC {
            return Err(Error::NotEncrypted);
        }
        let version = data[4];
        if version != VERSION {
            return Err(Error::UnsupportedVersion(version));
        }
        let kdf_id = data[5];
        if kdf_id != KDF_ARGON2ID {
            return Err(Error::UnsupportedKdf(kdf_id));
        }
        let cipher_id = data[6];
        if cipher_id != CIPHER_AES256GCM {
            return Err(Error::UnsupportedCipher(cipher_id));
        }
        let m_cost = u32::from_be_bytes(data[8..12].try_into().unwrap());
        let t_cost = u32::from_be_bytes(data[12..16].try_into().unwrap());
        let p_cost = u32::from_be_bytes(data[16..20].try_into().unwrap());
        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&data[20..36]);
        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&data[36..48]);
        Ok(Header {
            version,
            kdf_id,
            cipher_id,
            params: KdfParams {
                m_cost,
                t_cost,
                p_cost,
            },
            salt,
            nonce,
        })
    }
}
