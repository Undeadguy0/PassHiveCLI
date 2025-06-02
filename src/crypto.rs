use crate::db::models::{DataType, UserData};
use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordVerifier, Version,
    password_hash::{PasswordHasher, SaltString},
};
use chacha20poly1305::{Key, KeyInit, XChaCha20Poly1305, XNonce, aead::AeadMut};
use indicatif::*;
use rand::TryRngCore;
use rand_core::{OsRng, RngCore};
use serde_json::{Deserializer, Serializer};
use std::time::Duration;

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

pub fn encrypt_data(hash: &String, nonce: &[u8; 24], data: &String) -> Result<Vec<u8>, String> {
    let mut cipher = XChaCha20Poly1305::new(Key::from_slice(hash.as_bytes()));
    let cha_cha_nonce = XNonce::from_slice(nonce);
    match cipher.encrypt(cha_cha_nonce, data.as_ref()) {
        Err(_) => {
            return Err("Ошибка шифрования данных!".to_string());
        }
        Ok(encrypted) => {
            return Ok(encrypted);
        }
    }
}

pub fn unencrypt_data(hash: &String, nonce: &[u8; 24], data: &Vec<u8>) -> Result<DataType, String> {
    let mut cipher = XChaCha20Poly1305::new(Key::from_slice(hash.as_bytes()));
    let xnonce = XNonce::from_slice(nonce);
    match cipher.decrypt(&xnonce, data.as_ref()) {
        Err(_) => return Err("Ошибка расшифровки данных!!!".to_string()),
        Ok(unencrypted) => {
            let st = str::from_utf8(&unencrypted).unwrap();
            let total: DataType = serde_json::from_str(st).unwrap();
            return Ok(total);
        }
    }
}

pub fn unencrypt_str(hash: &String, nonce: &[u8; 24], string: &Vec<u8>) -> Result<String, String> {
    let mut cipher = XChaCha20Poly1305::new(Key::from_slice(hash.as_bytes()));
    let xnonce = XNonce::from_slice(nonce);
    match cipher.decrypt(&xnonce, string.as_ref()) {
        Err(_) => return Err("Ошибка расшифровки данных!!!".to_string()),
        Ok(uncrypt) => return Ok(str::from_utf8(&uncrypt).unwrap().to_string()),
    }
}
