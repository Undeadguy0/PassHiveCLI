use rusqlite::*;
use std::path::*;

use rusqlite::Connection;
use std::path::PathBuf;

pub fn init_db(path: &PathBuf) -> Result<(), String> {
    let db_path = path.join("passhive.db");

    let connection =
        Connection::open(db_path).map_err(|e| format!("Ошибка открытия базы данных: {}", e))?;

    let sql_meta = "
        CREATE TABLE IF NOT EXISTS meta (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            user_login TEXT NOT NULL,
            user_create_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            user_passwd TEXT NOT NULL,
            salt TEXT NOT NULL
        )";
    connection
        .execute(sql_meta, [])
        .map_err(|e| format!("Ошибка создания таблицы meta: {}", e))?;

    let sql_users = "
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            owner INTEGER NOT NULL,
            data_type TEXT NOT NULL,
            data TEXT NOT NULL
        )";
    connection
        .execute(sql_users, [])
        .map_err(|e| format!("Ошибка создания таблицы users: {}", e))?;
    Ok(())
}
