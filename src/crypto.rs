use crate::db::models::{DataType, UserData};
use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordVerifier, Version,
    password_hash::{PasswordHasher, SaltString},
};
use chacha20poly1305::{Key, KeyInit, XChaCha20Poly1305, XNonce, aead::AeadMut};
use indicatif::*;
use rand::TryRngCore;
use rand_core::{OsRng, RngCore};
use serde::Serialize;

use std::time::Duration;

fn vec_to_str(vec: &Vec<u8>) -> String {
    String::from_utf8(vec.clone()).unwrap()
}

pub fn encode(password: String) -> Result<(String, String), String> {
    let spinner = ProgressBar::new_spinner();
    let mut raw_salt: [u8; 32] = [0; 32];

    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&[
                "[    ]", "[=   ]", "[==  ]", "[=== ]", "[ ===]", "[  ==]", "[   =]", "[    ]",
            ])
            .template("{spinner} {msg}")
            .expect("Ошибка шаблона прогрессбара"),
    );
    spinner.set_message(
        "🐝Генерация хэша пароля... Не переживайте, обычно это занимает до 10 секунд!🐝",
    );
    spinner.enable_steady_tick(Duration::from_millis(120));

    if let Err(_) = OsRng.try_fill_bytes(&mut raw_salt) {
        return Err("Ошибка при генерации соли в битовом виде".to_string());
    }

    let salt = SaltString::encode_b64(&raw_salt);

    if let Err(_) = salt {
        return Err("Ошибка генерации соли".to_string());
    }

    let salt = salt.unwrap();

    let config = Params::new(65536, 10, 3, Some(64));

    if let Err(_) = config {
        return Err("Ошибка конфигуратора Argon2".to_string());
    }

    let config = config.unwrap();

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, config);

    let hash = argon2.hash_password(password.as_bytes(), &salt);

    if let Err(_) = hash {
        return Err("Ошибка хэширования".to_string());
    }

    let hash = hash.unwrap();

    Ok((hash.to_string(), salt.to_string()))
}

pub fn check_password(hash: &String, pass: &String) -> Result<bool, String> {
    match PasswordHash::new(hash) {
        Ok(parsed) => {
            let spinner = ProgressBar::new_spinner();
            spinner.set_style(
                ProgressStyle::default_spinner()
                    .tick_strings(&[
                        "[    ]", "[=   ]", "[==  ]", "[=== ]", "[ ===]", "[  ==]", "[   =]",
                        "[    ]",
                    ])
                    .template("{spinner} {msg}")
                    .expect("Ошибка шаблона прогрессбара"),
            );
            spinner.set_message(
                "🐝Сверяем пароль с хэшем... Не переживайте, обычно это занимает не более 10 секунд!🐝",
            );
            spinner.enable_steady_tick(Duration::from_millis(120));
            let argon2 = Argon2::default();
            return Ok(argon2.verify_password(pass.as_bytes(), &parsed).is_ok());
        }
        Err(e) => return Err(e.to_string()),
    }

    Err("Ошибка выхода из match в check_password!".to_string())
}

pub fn create_nonce() -> Result<[u8; 24], String> {
    let mut total = [0u8; 24];
    match OsRng.try_fill_bytes(&mut total) {
        Err(e) => return Err(e.to_string()),
        Ok(()) => return Ok(total),
    }
}

pub fn create_crypto_key(salt: &str, password: &str) -> Result<[u8; 32], String> {
    let argon2 = Argon2::default();
    let mut out = [0u8; 32];
    match argon2.hash_password_into(password.as_bytes(), salt.as_bytes(), &mut out) {
        Ok(()) => return Ok(out),
        Err(e) => return Err(e.to_string()),
    }
}

pub fn encrypt_str_with_nonce(
    string: &str,
    key: &[u8; 32],
    nonce: &[u8; 24],
) -> Result<Vec<u8>, String> {
    let mut cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    let xnonce = XNonce::from_slice(nonce);

    match cipher.encrypt(xnonce, string.as_bytes()) {
        Ok(enc) => return Ok(enc),
        Err(e) => Err(e.to_string()),
    }
}

pub fn encrypt_str(string: &str, key: &[u8; 32]) -> Result<(Vec<u8>, [u8; 24]), String> {
    let new_nonce;
    match create_nonce() {
        Ok(n) => new_nonce = n,
        Err(e) => return Err(e.to_string()),
    }

    let mut cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    let xnonce = XNonce::from_slice(&new_nonce);

    match cipher.encrypt(xnonce, string.as_bytes()) {
        Ok(enc) => return Ok((enc, new_nonce)),
        Err(e) => Err(e.to_string()),
    }
}

pub fn encrypt_data(data: &DataType, key: &[u8; 32]) -> Result<(Vec<u8>, [u8; 24]), String> {
    let serialised;
    match serde_json::to_string(data) {
        Ok(ser) => serialised = ser,
        Err(e) => return Err(e.to_string()),
    }

    match encrypt_str(serialised.as_str(), key) {
        Ok((enc, nonce)) => return Ok((enc, nonce)),
        Err(e) => return Err(e),
    }
}

pub fn decrypt_str(string: &Vec<u8>, nonce: &[u8; 24], key: &[u8; 32]) -> Result<String, String> {
    let mut cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    let xnonce = XNonce::from_slice(nonce);

    match cipher.decrypt(xnonce, string.as_ref()) {
        Ok(dec) => return Ok(vec_to_str(&dec)),
        Err(e) => return Err(e.to_string()),
    }
}

pub fn decrypt_data(data: &Vec<u8>, nonce: &[u8; 24], key: &[u8; 32]) -> Result<DataType, String> {
    let mut cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    let xnonce = XNonce::from_slice(nonce);

    match cipher.decrypt(xnonce, data.as_ref()) {
        Err(e) => return Err(e.to_string()),
        Ok(json_text) => {
            let json_string = vec_to_str(&json_text);
            return Ok(serde_json::from_str(json_string.as_str()).unwrap());
        }
    }
}
