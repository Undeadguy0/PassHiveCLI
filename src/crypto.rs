use core::prelude;
use std::{panic::AssertUnwindSafe, time::Duration};

use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordVerifier, Version,
    password_hash::{PasswordHasher, SaltString},
};
use indicatif::*;
use rand::TryRngCore;
use rand_core::{OsRng, RngCore};

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
