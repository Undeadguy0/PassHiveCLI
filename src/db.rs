use rusqlite::*;
use std::path::*;

pub fn init_db(path: &PathBuf) -> Result<Ok(), Err(String)> {
    let db_path = path.join("passhive.db");

    match Connection::open(db_path) {
        Ok(connection) => {
            connection.execute(
                "CREATE TABLE IF NOT EXISTS meta (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                user_login TEXT NOT NULL,
                user_create_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

                )",
            );
        }
        Err(err) => Err(format!("Ошибка создания базы данных: {}", err)),
    }
}
