//! Pure cryptographic core: Argon2id key derivation + AES-256-GCM AEAD.
//!
//! No file I/O happens here, which keeps it trivially unit-testable and
//! identical on desktop and mobile.

use crate::errors::Error;
use crate::format::{
    Header, KdfParams, CIPHER_AES256GCM, HEADER_LEN, KDF_ARGON2ID, KEY_LEN, NONCE_LEN, SALT_LEN,
    TAG_LEN, VERSION,
};
use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use zeroize::Zeroizing;

/// Fill a fixed-size buffer with cryptographically secure random bytes.
fn random<const N: usize>() -> Result<[u8; N], Error> {
    let mut buf = [0u8; N];
    getrandom::fill(&mut buf).map_err(|e| Error::Kdf(format!("csprng: {e}")))?;
    Ok(buf)
}

/// Derive a 256-bit key from `password` and `salt` using Argon2id.
/// The returned key is zeroized on drop.
pub fn derive_key(
    password: &[u8],
    salt: &[u8],
    params: &KdfParams,
) -> Result<Zeroizing<[u8; KEY_LEN]>, Error> {
    let argon2_params = Params::new(params.m_cost, params.t_cost, params.p_cost, Some(KEY_LEN))
        .map_err(|e| Error::Kdf(e.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    argon2
        .hash_password_into(password, salt, &mut key[..])
        .map_err(|e| Error::Kdf(e.to_string()))?;
    Ok(key)
}

fn seal(key: &[u8; KEY_LEN], header: &Header, plaintext: &[u8]) -> Result<Vec<u8>, Error> {
    let header_bytes = header.encode();
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| Error::Cipher(e.to_string()))?;
    let nonce = Nonce::try_from(&header.nonce[..])
        .map_err(|_| Error::Cipher("invalid nonce length".into()))?;
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext,
                aad: &header_bytes,
            },
        )
        .map_err(|e| Error::Cipher(e.to_string()))?;
    let mut out = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    out.extend_from_slice(&header_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Encrypt `plaintext` under `password` with a fresh salt+nonce.
/// Returns `(file_bytes, salt, key)` so the caller can cache the key and salt
/// for subsequent in-place saves without asking for the password again.
pub fn encrypt_new(
    plaintext: &[u8],
    password: &[u8],
    params: KdfParams,
) -> Result<(Vec<u8>, [u8; SALT_LEN], Zeroizing<[u8; KEY_LEN]>), Error> {
    let salt = random::<SALT_LEN>()?;
    let nonce = random::<NONCE_LEN>()?;
    let key = derive_key(password, &salt, &params)?;
    let header = Header {
        version: VERSION,
        kdf_id: KDF_ARGON2ID,
        cipher_id: CIPHER_AES256GCM,
        params,
        salt,
        nonce,
    };
    let bytes = seal(&key, &header, plaintext)?;
    Ok((bytes, salt, key))
}

/// Decrypt `data` with `password`. Returns `(plaintext, header, key)`.
pub fn decrypt(
    data: &[u8],
    password: &[u8],
) -> Result<(Vec<u8>, Header, Zeroizing<[u8; KEY_LEN]>), Error> {
    let header = Header::decode(data)?;
    if data.len() < HEADER_LEN + TAG_LEN {
        return Err(Error::BadPassword);
    }
    let key = derive_key(password, &header.salt, &header.params)?;
    let aad = &data[..HEADER_LEN];
    let ciphertext = &data[HEADER_LEN..];
    let cipher = Aes256Gcm::new_from_slice(&key[..]).map_err(|e| Error::Cipher(e.to_string()))?;
    let nonce = Nonce::try_from(&header.nonce[..])
        .map_err(|_| Error::Cipher("invalid nonce length".into()))?;
    let plaintext = cipher
        .decrypt(
            &nonce,
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map_err(|_| Error::BadPassword)?;
    Ok((plaintext, header, key))
}

/// Re-encrypt `plaintext` with an already-derived `key` and its `salt`,
/// generating a fresh nonce (never reuse a nonce with the same key).
pub fn reseal(
    key: &[u8; KEY_LEN],
    salt: [u8; SALT_LEN],
    params: KdfParams,
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    let nonce = random::<NONCE_LEN>()?;
    let header = Header {
        version: VERSION,
        kdf_id: KDF_ARGON2ID,
        cipher_id: CIPHER_AES256GCM,
        params,
        salt,
        nonce,
    };
    seal(key, &header, plaintext)
}

/// Structural probe: is `data` plausibly a SafeText container?
/// Does not verify the password.
pub fn probe(data: &[u8]) -> bool {
    Header::decode(data).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fast_params() -> KdfParams {
        KdfParams {
            m_cost: 8,
            t_cost: 1,
            p_cost: 1,
        }
    }

    const PW: &[u8] = b"correct horse battery staple";

    #[test]
    fn roundtrip() {
        let pt = "你好，世界\nhello & <xml>\n".as_bytes();
        let (data, salt, key) = encrypt_new(pt, PW, fast_params()).unwrap();
        assert!(data.len() > HEADER_LEN);
        assert_eq!(&data[0..4], b"STXT");
        assert!(probe(&data));

        let (got, header, _k) = decrypt(&data, PW).unwrap();
        assert_eq!(got, pt);
        assert_eq!(header.salt, salt);

        // fresh nonce -> different ciphertext for same plaintext
        let (data2, _, _) = encrypt_new(pt, PW, fast_params()).unwrap();
        assert_ne!(data, data2);

        // reseal with cached key reuses salt but must produce new bytes
        let data3 = reseal(&key, salt, fast_params(), b"second revision").unwrap();
        assert_ne!(data, data3);
        let (got3, _h, _) = decrypt(&data3, PW).unwrap();
        assert_eq!(got3, b"second revision");
    }

    #[test]
    fn wrong_password_rejected() {
        let (data, _, _) = encrypt_new(b"secret", PW, fast_params()).unwrap();
        assert_eq!(
            decrypt(&data, b"wrong password").unwrap_err(),
            Error::BadPassword
        );
    }

    #[test]
    fn tampered_ciphertext_rejected() {
        let (mut data, _, _) = encrypt_new(b"secret", PW, fast_params()).unwrap();
        let n = data.len();
        data[n - 1] ^= 0x01;
        assert_eq!(decrypt(&data, PW).unwrap_err(), Error::BadPassword);
    }

    #[test]
    fn tampered_header_rejected() {
        let (mut data, _, _) = encrypt_new(b"secret", PW, fast_params()).unwrap();
        data[5] = 9; // kdf_id
        assert_eq!(decrypt(&data, PW).unwrap_err(), Error::UnsupportedKdf(9));

        let (mut data2, _, _) = encrypt_new(b"secret", PW, fast_params()).unwrap();
        data2[4] = 99; // version
        assert_eq!(
            decrypt(&data2, PW).unwrap_err(),
            Error::UnsupportedVersion(99)
        );
    }

    #[test]
    fn tampered_salt_is_detected_as_bad_password() {
        // Changing the salt changes the derived key, so decryption fails auth.
        let (mut data, _, _) = encrypt_new(b"secret", PW, fast_params()).unwrap();
        data[20] ^= 0x01;
        assert_eq!(decrypt(&data, PW).unwrap_err(), Error::BadPassword);
    }

    #[test]
    fn plaintext_and_truncated_rejected() {
        assert_eq!(
            decrypt(b"just a normal txt file", PW).unwrap_err(),
            Error::NotEncrypted
        );
        let (data, _, _) = encrypt_new(b"secret", PW, fast_params()).unwrap();
        for cut in [0usize, 1, 10, HEADER_LEN - 1] {
            assert_eq!(
                decrypt(&data[..cut], PW).unwrap_err(),
                Error::NotEncrypted,
                "cut={cut}"
            );
        }
        // header intact but ciphertext/tag truncated
        let short = &data[..HEADER_LEN + TAG_LEN - 1];
        assert_eq!(decrypt(short, PW).unwrap_err(), Error::BadPassword);
    }

    #[test]
    fn probe_detects_magic() {
        assert!(!probe(b"hello"));
        assert!(!probe(&[0u8; 100]));
        let (data, _, _) = encrypt_new(b"x", PW, fast_params()).unwrap();
        assert!(probe(&data));
    }

    #[test]
    fn empty_plaintext_roundtrip() {
        let (data, _, _) = encrypt_new(b"", PW, fast_params()).unwrap();
        let (got, _h, _) = decrypt(&data, PW).unwrap();
        assert!(got.is_empty());
    }
}
