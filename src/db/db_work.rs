use super::models::*;
use rusqlite::{Connection, params};
use std::path::PathBuf;

fn connect_to_db(path: &PathBuf) -> Result<Connection, String> {
    Connection::open(path.join("passhive.db"))
        .map_err(|e| format!("Ошибка открытия базы данных: {}", e))
}

pub fn init_db(path: &PathBuf) -> Result<(), String> {
    let connection;

    match connect_to_db(path) {
        Ok(conn) => connection = conn,
        Err(e) => return Err(e),
    }

    let sql_meta = "
        CREATE TABLE IF NOT EXISTS meta (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
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

pub fn users_empty(path: &PathBuf) -> bool {
    let connection;

    match connect_to_db(path) {
        Ok(conn) => connection = conn,
        Err(_) => unreachable!("БД ПРОПАЛА ПО ХОДУ РАБОТЫ!"),
    }

    let sql = "SELECT COUNT(*) FROM meta";
    let count: i64 = connection.query_row(sql, [], |row| row.get(0)).unwrap_or(0);

    count == 0
}

pub fn user_exists(path: &PathBuf, login: &str) -> Result<bool, String> {
    let conn;

    match connect_to_db(path) {
        Ok(con) => conn = con,
        Err(e) => return Err(e),
    }

    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM meta WHERE user_login = ?1")
        .map_err(|e| e.to_string())?;

    let count: i64 = stmt
        .query_row(params![login], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    Ok(count > 0)
}

pub fn reg_new_user(path: &PathBuf, login: &str, hash: &str, salt: &str) -> Result<i64, String> {
    let connection;

    match connect_to_db(path) {
        Ok(conn) => connection = conn,
        Err(e) => return Err(e),
    }

    let sql = "INSERT INTO meta (user_login, user_passwd, salt) VALUES (?1, ?2, ?3)";
    connection
        .execute(sql, params![login, hash, salt])
        .map_err(|e| format!("Ошибка регистрации нового пользователя в БД: {}", e))?;

    Ok(connection.last_insert_rowid())
}

pub fn find_by_login(path: &PathBuf, login: &String) -> Result<Option<(i64, String)>, String> {
    let connection = connect_to_db(path).map_err(|e| e.to_string())?;

    let sql = "SELECT id, user_passwd FROM meta WHERE user_login = ?1";

    let mut stmt = connection.prepare(sql).map_err(|e| e.to_string())?;

    let result = stmt.query_row(params![login], |row| {
        let id: i64 = row.get(0)?;
        let passwd: String = row.get(1)?;
        Ok((id, passwd))
    });

    match result {
        Ok(data) => Ok(Some(data)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
