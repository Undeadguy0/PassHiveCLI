mod cli;
mod crypto;
mod db;
mod os_work;
use crate::{
    cli::throw_err,
    crypto::{create_crypto_key, encrypt_str_with_nonce},
    db::{db_work::*, models::*},
};
use colored::Colorize;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use db::{db_work, models};
use std::{collections::BTreeMap, path::PathBuf};

pub struct ShowableData {
    pub id: i64,
    pub name: String,
    pub notice: String,
    pub data: models::DataType,
}

fn init_user_data(path: &PathBuf, id: i64, key: &[u8; 32]) -> BTreeMap<String, Vec<ShowableData>> {
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
        let decrypted = crypto::decrypt_data(&data.data, &data.nonce, key);
        match decrypted {
            Err(e) => cli::throw_err(e),
            Ok(decrypted_data) => {
                let mut entry = String::new();
                let check_type = data.data_type.clone();
                match check_type {
                    DataType::Card { .. } => entry = "card".to_string(),
                    DataType::Token { .. } => entry = "token".to_string(),
                    DataType::Passport { .. } => entry = "passport".to_string(),
                    DataType::WifiConfig { .. } => entry = "wificonfig".to_string(),
                    DataType::Password { .. } => entry = "password".to_string(),
                    DataType::Document { .. } => entry = "document".to_string(),
                }
                match crypto::decrypt_str(&data.name, &data.nonce, key) {
                    Ok(nam) => match crypto::decrypt_str(&data.notice, &data.nonce, key) {
                        Ok(not) => total.get_mut(&entry).unwrap().push(ShowableData {
                            id: data.id,
                            name: nam,
                            notice: not,
                            data: decrypted_data,
                        }),
                        Err(e) => cli::throw_err(e),
                    },
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

fn auth(path: &PathBuf) -> (i64, [u8; 32]) {
    //ID в БД + ключ
    loop {
        print!("\x1B[2J\x1B[1;1H");
        let (input_login, input_password) = cli::get_auth_data(&path);

        match db_work::find_by_login(path, &input_login) {
            Err(e) => cli::throw_err(e),
            Ok(response) => match response {
                Some((db_id, hash)) => match crypto::check_password(&hash, &input_password) {
                    Err(e) => cli::throw_err(e),
                    Ok(is_correct) => {
                        if is_correct {
                            cli::auth_seccess();
                            match db_work::get_salt_by_id(path, db_id) {
                                Err(e) => {
                                    cli::throw_err(e);
                                }
                                Ok(salt) => {
                                    match create_crypto_key(salt.as_str(), input_password.as_str())
                                    {
                                        Err(e) => cli::throw_err(e),
                                        Ok(key) => return (db_id, key),
                                    }
                                }
                            }
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

fn add_row_mode(
    path: &PathBuf,
    id: i64,
    key: &[u8; 32],
    all_rows: &mut BTreeMap<String, Vec<ShowableData>>,
) {
    let new_row;
    match cli::get_new_row_data() {
        Err(e) => {
            cli::throw_err(e);
            return;
        }
        Ok(row) => new_row = row,
    }

    match crypto::encrypt_data(&new_row.data, key) {
        Err(e) => cli::throw_err(e),
        Ok((enc_data, nonce)) => {
            match crypto::encrypt_str_with_nonce(&new_row.name.as_str(), key, &nonce) {
                Err(e) => cli::throw_err(e),
                Ok(enc_name) => {
                    match encrypt_str_with_nonce(&new_row.notice.as_str(), key, &nonce) {
                        Err(e) => cli::throw_err(e),
                        Ok(enc_notice) => match db_work::insert_row(
                            path,
                            id,
                            &new_row.data.formal_name(),
                            &enc_data,
                            &enc_name,
                            &enc_notice,
                            &nonce,
                        ) {
                            Err(e) => cli::throw_err(e),
                            Ok(fresh_id) => {
                                all_rows.get_mut(&new_row.data.formal_name()).unwrap().push(
                                    ShowableData {
                                        id: fresh_id,
                                        name: new_row.name,
                                        notice: new_row.notice,
                                        data: new_row.data,
                                    },
                                );
                            }
                        },
                    }
                }
            }
        }
    }
}
fn update_row_mode(
    path: &PathBuf,
    id: i64,
    key: &[u8; 32],
    all_rows: &mut BTreeMap<String, Vec<ShowableData>>,
) {
    let (db_id, (partision_index, local_index)) = cli::select_id_to_update(all_rows);
    let mut target: Option<&mut ShowableData> = None;

    for part in all_rows.iter_mut().enumerate() {
        if part.0 == partision_index {
            target = Some(
                part.1
                    .1
                    .get_mut(local_index)
                    .expect("Ошибка... каким-то образом данных с таким индексом нет"),
            );
            break;
        }
    }

    let free_target = target.expect("Каким-то образом ненайденный id прошел мимо expect ранее");

    let updated_data = cli::correct_data(free_target);

    match crypto::encrypt_data(&updated_data.data, key) {
        Err(e) => throw_err(e),
        Ok((encrypted, nonce)) => {
            match crypto::encrypt_str_with_nonce(&updated_data.name, key, &nonce) {
                Err(e) => throw_err(e),
                Ok(enc_name) => {
                    match crypto::encrypt_str_with_nonce(&updated_data.notice, key, &nonce) {
                        Err(e) => throw_err(e),
                        Ok(enc_notice) => {
                            match update_row(path, encrypted, nonce, enc_name, enc_notice, db_id) {
                                Err(e) => throw_err(e),
                                Ok(()) => {
                                    *free_target = updated_data;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
fn delete_row_mode(path: &PathBuf, all_rows: &mut BTreeMap<String, Vec<ShowableData>>) {
    let (db_id, (partision_index, local_index)) = cli::select_id_to_delete(all_rows);
    db_work::delete_row(path, db_id).expect("Ошибка удаления из БД");

    for part in all_rows.iter_mut().enumerate() {
        if part.0 == partision_index {
            part.1.1.remove(local_index);
            return;
        }
    }
}

pub fn update_or_save<T>(first: Option<T>, second: T) -> T {
    //Если первый не None - возвращает первый аргумент. Иначе второй
    if let None = first {
        return second;
    }
    first.unwrap()
}

fn main() {
    let default_style = cli::TableStyle::new(
        "=".to_string().truecolor(255, 255, 255),
        "-".to_string().truecolor(255, 255, 255),
        "|".to_string().truecolor(255, 255, 255),
        "#".to_string().truecolor(255, 255, 255),
        "~".to_string().truecolor(255, 255, 255),
        (66, 250, 20),
        (255, 255, 255),
    );

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

    let mut main_user_id = -1; //ID вошедшего пользователя
    let mut main_key = [0u8; 32]; // Основной ключ для шифрования
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
                    (main_user_id, main_key) = auth(&path);
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
    if main_user_id == -1 {
        cli::throw_err(
            "Ошибка выхода из цикла входа + регистрации, id пользователя - -1!!!".to_string(),
        );
    }

    let mut global_user_data = init_user_data(&path, main_user_id, &main_key);

    loop {
        if let Err(e) = enable_raw_mode() {
            cli::throw_err(e.to_string());
        }

        print!("\x1B[2J\x1B[1;1H");
        disable_raw_mode().expect("Ошибка выхода из сырого режима!");
        // cli::show_all_data(&global_user_data); минималистичная функция для просмотра записей, включать только для дебага добавления записей
        enable_raw_mode().expect("Ошибка входа в сырой режим!");
        cli::show_hotkeys();
        cli::show_data_extended(&global_user_data, &default_style);
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
                        add_row_mode(&path, main_user_id, &main_key, &mut global_user_data);
                    }
                    (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                        update_row_mode(&path, main_user_id, &main_key, &mut global_user_data);
                    }
                    (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                        delete_row_mode(&path, &mut global_user_data);
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

    disable_raw_mode().unwrap();
}
