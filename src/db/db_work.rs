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
            data BLOB NOT NULL,
            name BLOB,
            notice BLOB,
            nonce BLOB NOT NULL
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

pub fn get_all_user_data(path: &PathBuf, user_id: i64) -> Result<Vec<UserData>, rusqlite::Error> {
    let connection = connect_to_db(path).map_err(|e| rusqlite::Error::InvalidQuery)?;

    let mut stmt = connection
        .prepare("SELECT id, data_type, data, name, notice, nonce FROM users WHERE owner = ?1")?;

    let user_iter = stmt.query_map(params![user_id], |row| {
        let id = row.get::<_, i64>(0)?;
        let data_type = row.get::<_, String>(1)?;
        let data = row.get::<_, Vec<u8>>(2)?;
        let name = row.get::<_, Vec<u8>>(3)?;
        let notice = row.get::<_, Vec<u8>>(4)?;
        let nonce = row.get::<_, [u8; 24]>(5)?;
        Ok(UserData::new(id, data, data_type, nonce, notice, name))
    })?;

    user_iter.collect()
}

pub fn insert_row(
    path: &PathBuf,
    owner: i64,
    data_type: &String,
    data: &Vec<u8>,
    name: &Vec<u8>,
    notice: &Vec<u8>,
    nonce: &[u8; 24],
) -> Result<i64, String> {
    let connection = connect_to_db(path).map_err(|e| format!("Ошибка подключения к БД: {}", e))?;

    let sql = "INSERT INTO users (owner, data_type, data, name, notice, nonce) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";

    match connection.execute(sql, params![owner, data_type, data, name, notice, nonce]) {
        Ok(_) => {
            let id = connection.last_insert_rowid();
            Ok(id)
        }
        Err(e) => Err(format!("Ошибка вставки данных: {}", e)),
    }
}

pub fn get_salt_by_id(path: &PathBuf, id: i64) -> Result<String, String> {
    let connection = connect_to_db(path).map_err(|e| format!("Ошибка подключения к БД: {}", e))?;

    let sql = "SELECT salt FROM meta WHERE id = ?1";
    let mut stmt = connection
        .prepare(sql)
        .map_err(|e| format!("Ошибка подготовки запроса: {}", e))?;

    stmt.query_row(params![id], |row| row.get::<_, String>(0))
        .map_err(|_| {
            "Ошибка: нет пользователя с таким ID или проблемы с чтением строки".to_string()
        })
}

pub fn delete_row(path: &PathBuf, id: i64) -> Result<(), String> {
    let connection = connect_to_db(path).map_err(|e| format!("Ошибка подключения к БД: {}", e))?;

    let sql = "DELETE FROM users WHERE id=?1";

    match connection.execute(sql, params![id]) {
        Err(e) => return Err(e.to_string()),
        Ok(_) => return Ok(()),
    }
}

pub fn update_row(
    path: &PathBuf,
    enc_data: Vec<u8>,
    nonce: [u8; 24],
    name: Vec<u8>,
    notice: Vec<u8>,
    id: i64,
) -> Result<(), String> {
    let connection = connect_to_db(path).map_err(|e| format!("Ошибка подключения к БД: {}", e))?;

    let sql = "
        UPDATE users
        SET data = ?1, name = ?2, notice = ?3, nonce = ?4
        WHERE id = ?5";
    match connection.execute(sql, params![enc_data, name, notice, nonce, id]) {
        Err(e) => Err(e.to_string()),
        Ok(_) => Ok(()),
    }
}
