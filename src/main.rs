mod cli;
mod crypto;
mod db;
mod os_work;
use argon2::password_hash;
use cli::db_conn_success;
use colored::Colorize;
use db::db_work;
use std::path::PathBuf;

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

    let mut global_id;
    let mut global_hash;
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
}
