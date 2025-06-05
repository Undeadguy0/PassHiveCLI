mod cli;
mod crypto;
mod db;
mod os_work;
use colored::Colorize;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use db::{db_work, models};
use std::{collections::BTreeMap, path::PathBuf};

use crate::db::models::DataType;

pub struct ShowableData {
    pub id: i64,
    pub name: String,
    pub nonce: [u8; 24],
    pub data: models::DataType,
}

fn init_user_data(
    global_hash: &String,
    path: &PathBuf,
    id: i64,
) -> BTreeMap<String, Vec<ShowableData>> {
    let mut uploaded_data = vec![];
    match db_work::get_all_user_data(path, id) {
        Err(e) => cli::throw_err(e.to_string()),
        Ok(data) => uploaded_data = data,
    }

    let mut total: BTreeMap<String, Vec<ShowableData>> = BTreeMap::new();

    total.insert("password".to_string(), Vec::new());
    total.insert("card".to_string(), Vec::new());
    total.insert("document".to_string(), Vec::new());
    total.insert("token".to_string(), Vec::new());
    total.insert("wificonfig".to_string(), Vec::new());
    total.insert("passport".to_string(), Vec::new());

    for data in uploaded_data.iter() {
        let decrypted = crypto::unencrypt_data(global_hash, &(data.nonce), &(data.data));
        match decrypted {
            Err(e) => cli::throw_err(e),
            Ok(decrypted_data) => {
                let mut entry = String::new();
                let check_type = data.data_type.clone();
                match check_type {
                    DataType::Card { num, cvv, bank } => entry = "card".to_string(),
                    DataType::Token { token, from } => entry = "token".to_string(),
                    DataType::Passport {
                        fsl,
                        date,
                        sex,
                        serial,
                        num,
                    } => entry = "passport".to_string(),
                    DataType::WifiConfig { name, password } => entry = "wificonfig".to_string(),
                    DataType::Password { password } => entry = "password".to_string(),
                    DataType::Document { text } => entry = "document".to_string(),
                }
                match crypto::unencrypt_str(global_hash, &(data.nonce), &(&data.name)) {
                    Ok(n) => {
                        total.get_mut(&entry).unwrap().push(ShowableData {
                            id: data.id,
                            name: n,
                            nonce: data.nonce,
                            data: decrypted_data,
                        });
                    }
                    Err(e) => cli::throw_err(e),
                }
            }
        }
    }

    total
}

fn reg(path: &PathBuf) -> bool {
    let (login, password) = cli::registration();

    match db_work::user_exists(path, &login) {
        Err(e) => cli::throw_err(e),
        Ok(b) => {
            if b {
                cli::user_exists_err();
                return false;
            }
        }
    }
    match crypto::encode(password) {
        Ok((hash, salt)) => {
            cli::regist_success(&login);

            match db_work::reg_new_user(&path, &login, &hash, &salt) {
                Ok(row) => {
                    cli::registration_success(&login);
                    return true;
                }
                Err(e) => cli::throw_err(e),
            }
        }
        Err(e) => cli::throw_err(e),
    }

    unreachable!("Ошибка в цикле регистрации!");
}

fn auth(path: &PathBuf) -> (i64, String) {
    loop {
        print!("\x1B[2J\x1B[1;1H");
        let (input_login, input_password) = cli::get_auth_data(&path);

        match db_work::find_by_login(&path, &input_login) {
            Err(e) => cli::throw_err(e),
            Ok(response) => match response {
                Some((db_id, hash)) => match crypto::check_password(&hash, &input_password) {
                    Err(e) => cli::throw_err(e),
                    Ok(is_correct) => {
                        if is_correct {
                            cli::auth_seccess();
                            return (db_id, hash);
                        } else {
                            cli::auth_failure();
                            continue;
                        }
                    }
                },
                _ => {
                    println!(
                        "{}",
                        "Ошибка аутентификации. Попробуйте еще раз.".purple().bold()
                    );
                    continue;
                }
            },
        }
    }
}

fn add_row_mode(path: &PathBuf, id: i64, hash: &String) {
    cli::get_new_row_data();
}
fn update_row_mode(path: &PathBuf, id: i64, hash: &String) {}
fn delete_row_mode(path: &PathBuf, id: i64, hash: &String) {}

fn main() {
    cli::hi();

    let (response, path, os) = os_work::verify_data();

    if !response {
        cli::check_error(&os);
        if let Err(err) = os_work::init_dir(&path) {
            cli::throw_err(err.to_string());
        }
        cli::success_init_dir(&path);
    } else {
        cli::check_success();
    }

    if os_work::check_exists(&path, "passhive.db") {
        cli::db_conn_success();
    } else {
        if let Err(err) = db_work::init_db(&path) {
            cli::throw_err(err.to_string());
        }
        cli::success_init_db();
    }

    let mut global_id = -1;
    let mut global_hash = String::default();
    let mut exit = false;

    loop {
        if db_work::users_empty(&path) {
            if !reg(&path) {
                continue;
            }
        } else {
            match cli::log_or_reg() {
                cli::AccountManipulation::Regist => {
                    if !reg(&path) {
                        continue;
                    }
                }
                cli::AccountManipulation::Auth => {
                    (global_id, global_hash) = auth(&path);
                    exit = true;
                    break;
                }
            }
        }
        if exit {
            break;
        }
    }

    // вход прошел успешно - основной цикл
    if global_id == -1 {
        cli::throw_err(
            "Ошибка выхода из цикла входа + регистрации, id пользователя - -1!!!".to_string(),
        );
    }

    let mut global_user_data = init_user_data(&global_hash, &path, global_id);
    loop {
        if let Err(e) = enable_raw_mode() {
            cli::throw_err(e.to_string());
        }

        print!("\x1B[2J\x1B[1;1H");
        cli::show_all_data(&global_user_data);
        cli::show_hotkeys();
        loop {
            if let Err(_) = event::poll(std::time::Duration::from_millis(500)) {
                cli::throw_err("Ошибка обработки событий!".to_string());
            } else {
                if let Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind,
                    state,
                }) = event::read().unwrap()
                {
                    match (code, modifiers) {
                        (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                            add_row_mode(&path, global_id, &global_hash);
                        }
                        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                            update_row_mode(&path, global_id, &global_hash);
                        }
                        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                            delete_row_mode(&path, global_id, &global_hash);
                        }
                        (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                            disable_raw_mode().unwrap();
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
            }
        }
        break;
    }

    disable_raw_mode().unwrap();
}
