// Copyright 2025 Aquila Labs of Alberta, Canada <matt@cicero.sh>
// Licensed under either the Apache License, Version 2.0 OR the MIT License, at your option.
// You may not use this file except in compliance with one of the Licenses.
// Apache License text: https://www.apache.org/licenses/LICENSE-2.0
// MIT License text: https://opensource.org/licenses/MIT

use crate::Error;
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key};
use argon2::Argon2;
use bip39::Mnemonic;
use hkdf::Hkdf;
use sha2::{Digest, Sha256};
use std::convert::TryInto;
use std::fs;

const PREFIX: u8 = 0x43;
const VERSION: u8 = 0x01;

/// Encrypts a message using AES-256-GCM with a password-derived key.
/// Returns a concatenated blob: [header | encrypted_key | iv | nonce | salt | ciphertext].
pub fn encrypt(message: &[u8], password: [u8; 32]) -> Result<Vec<u8>, Error> {
    let mut rng = OsRng;

    // Generate encryption key
    let mut encryption_key = [0u8; 32];
    rng.fill_bytes(&mut encryption_key);

    // Encrypt
    let output = encrypt_with_master_key(message, password, encryption_key)?;

    Ok(output)
}

/// Encrypt with master key
pub fn encrypt_with_master_key(
    message: &[u8],
    password: [u8; 32],
    master_key: [u8; 32],
) -> Result<Vec<u8>, Error> {
    let mut rng = OsRng;

    // Generate iv
    let mut iv = [0u8; 12];
    rng.fill_bytes(&mut iv);

    // Encrypt message
    let key: &Key<Aes256Gcm> = &master_key.into();
    let cipher = Aes256Gcm::new(key);
    let ciphertext =
        cipher.encrypt(&iv.into(), message.as_ref()).map_err(|e| Error::Crypto(e.to_string()))?;

    // Get password iv
    let mut password_iv = [0u8; 12];
    rng.fill_bytes(&mut password_iv);

    // Derive child / specific message encryption key
    let (argon_hash, salt) = argon2_hash(&password, None)?;
    let (child_key, nonce) = derive_key(&argon_hash, None)?;

    // Encrypt outer seal
    let outer_key = Key::<Aes256Gcm>::from(child_key);
    let outer_cipher = Aes256Gcm::new(&outer_key);
    let encrypted_full_key = outer_cipher
        .encrypt(&password_iv.into(), master_key.as_ref())
        .map_err(|e| Error::Crypto(e.to_string()))?;

    // Put it all together
    let mut header = vec![PREFIX, VERSION];
    header.extend_from_slice(&encrypted_full_key);
    header.extend_from_slice(&iv);
    header.extend_from_slice(&password_iv);
    header.extend_from_slice(&nonce);
    header.extend_from_slice(&salt);

    Ok([header, ciphertext].concat())
}

/// Encrypts a message using a string password, normalized to 32 bytes via SHA-256.
pub fn _encrypt_with_str(message: &[u8], password: &str) -> Result<Vec<u8>, Error> {
    encrypt(message, normalize_password(password))
}

/// Decrypts a payload encrypted with `encrypt`.
/// Returns the plaintext or an error if the prefix, version, or password is invalid.
pub fn decrypt(payload: &[u8], password: [u8; 32]) -> Result<Vec<u8>, Error> {
    // Extract master key
    let (iv, msg_key) = extract_master_key(payload, password)?;

    // Decrypt message
    let msg_cipher = Aes256Gcm::new(&msg_key.into());
    let message = msg_cipher
        .decrypt(&iv.into(), payload[122..].as_ref())
        .map_err(|_| Error::Crypto("Invalid dencryption password.".to_string()))?;

    Ok(message)
}

/// Extract master key
pub fn extract_master_key(
    payload: &[u8],
    password: [u8; 32],
) -> Result<([u8; 12], [u8; 32]), Error> {
    // Check header
    if payload.len() < 122 {
        return Err(Error::Crypto("Payload too short".to_string()));
    }
    if payload[0] != PREFIX || payload[1] != VERSION {
        return Err(Error::Crypto("Invalid prefix or version".to_string()));
    }

    // Define empty arrays
    let mut password_iv: [u8; 12] = [0; 12];
    let mut nonce: [u8; 32] = [0; 32];
    let mut salt: [u8; 16] = [0; 16];

    // Get password iv, nonce, and salt
    password_iv.copy_from_slice(&payload[62..74]);
    nonce.copy_from_slice(&payload[74..106]);
    salt.copy_from_slice(&payload[106..122]);

    // Argon2 hash and derive child
    let (argon_hash, _) = argon2_hash(&password, Some(salt))?;
    let (child_key, _) = derive_key(&argon_hash, Some(nonce))?;
    let key = Key::<Aes256Gcm>::from_slice(&child_key);

    // Decrypd seal
    let cipher = Aes256Gcm::new(key);
    let inner_seal = cipher
        .decrypt(&password_iv.into(), payload[2..50].as_ref())
        .map_err(|_| Error::Crypto("Invalid encryption key.".to_string()))?;

    // Get iv and encryption key
    let mut iv: [u8; 12] = [0; 12];
    iv.copy_from_slice(&payload[50..62]);
    let mut msg_key: [u8; 32] = [0; 32];
    msg_key.copy_from_slice(&inner_seal[0..32]);

    Ok((iv, msg_key))
}

/// Save an existing file while retaining the same master key for password recovery via BIP39
pub fn update_existing_file(
    filepath: &str,
    payload: &[u8],
    n_password: [u8; 32],
) -> Result<(), Error> {
    // Get file
    let encrypted_bytes = fs::read(filepath)?;

    // Get master key
    let (_, master_key) = extract_master_key(&encrypted_bytes, n_password)?;

    // Encrypt with password
    let bytes = encrypt_with_master_key(payload, n_password, master_key)?;

    // Save file
    fs::write(filepath, &bytes)?;
    Ok(())
}

/// Decrypts a payload using a string password, normalized to 32 bytes.
pub fn _decrypt_with_str(payload: &[u8], password: &str) -> Result<Vec<u8>, Error> {
    decrypt(payload, normalize_password(password))
}

/// Ensure password is  32 byte array
pub fn normalize_password(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.finalize().into()
}

/// Hash via Argon2
fn argon2_hash(
    password: &[u8; 32],
    previous_salt: Option<[u8; 16]>,
) -> Result<([u8; 32], [u8; 16]), Error> {
    // Check if we have salt
    let mut salt: [u8; 16] = [0; 16];
    if let Some(prev_salt) = previous_salt {
        salt = prev_salt;
    } else {
        let mut rng = OsRng;
        rng.fill_bytes(&mut salt);
    }

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(65536, 2, 4, None).map_err(|e| Error::Crypto(e.to_string()))?,
    );

    let mut hash = [0u8; 32];
    argon2
        .hash_password_into(password, &salt, &mut hash)
        .map_err(|e| Error::Crypto(e.to_string()))?;

    Ok((hash, salt))
}

// Derive child
fn derive_key(
    password: &[u8; 32],
    previous_nonce: Option<[u8; 32]>,
) -> Result<([u8; 32], [u8; 32]), Error> {
    // Generate nonce
    let nonce: [u8; 32] = get_nonce(previous_nonce);

    // Derive child key from nonce
    let mut child_bytes = [0u8; 32];
    Hkdf::<Sha256>::from_prk(password.as_ref())
        .map_err(|e| Error::Crypto(e.to_string()))?
        .expand(&nonce, &mut child_bytes)
        .map_err(|e| Error::Crypto(e.to_string()))?;

    Ok((child_bytes, nonce))
}

/// Generates BIP-39 mnemonic words from a password-derived entropy.
pub fn get_bip39_words(payload: &[u8], password: &str) -> Result<Vec<String>, Error> {
    let n_password = normalize_password(password);
    let (_, master_key) = extract_master_key(payload, n_password)?;

    let mnemonic = Mnemonic::from_entropy(&master_key).expect("Invalid entropy");
    Ok(mnemonic.words().map(String::from).collect())
}

/// Restore from BIP39 words
pub fn restore_from_bip39_words(
    payload: &[u8],
    phrase: &str,
) -> Result<(Vec<u8>, [u8; 32]), Error> {
    // Check header
    if payload.len() < 122 {
        return Err(Error::Crypto("Payload too short".to_string()));
    }
    if payload[0] != PREFIX || payload[1] != VERSION {
        return Err(Error::Crypto("Invalid prefix or version".to_string()));
    }

    // Get iv and encryption key
    let mut iv: [u8; 12] = [0; 12];
    iv.copy_from_slice(&payload[50..62]);

    // Get the master key
    let mnemonic = Mnemonic::parse(phrase)
        .map_err(|e| Error::Crypto(format!("Unable to convert phrase to master key: {}", e)))?;
    let entropy: &[u8] = &mnemonic.to_entropy();
    let master_key: [u8; 32] = entropy
        .try_into()
        .map_err(|e| Error::Generic(format!("Unable to convert master key to 32 bytes: {}", e)))?;

    // Decrypt message
    let msg_cipher = Aes256Gcm::new(&master_key.into());
    let message = msg_cipher
        .decrypt(&iv.into(), payload[122..].as_ref())
        .map_err(|_| Error::Crypto("Invalid dencryption password.".to_string()))?;

    Ok((message, master_key))
}

pub fn get_nonce(previous_nonce: Option<[u8; 32]>) -> [u8; 32] {
    // Generate nonce
    let mut nonce: [u8; 32] = [0; 32];
    if let Some(prev_nonce) = previous_nonce {
        nonce = prev_nonce;
    } else {
        let mut rng = OsRng;
        rng.fill_bytes(&mut nonce);
    }

    nonce
}
