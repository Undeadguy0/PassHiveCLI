use std::default;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Serialize, Deserialize)]
struct UserMeta {
    id: i64,
    login: String,
    created_at: String,
    hash: String,
    salt: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Sex {
    Male,
    Female,
    NotDeclaredYet,
}
impl Sex {
    pub fn to_string(&self) -> String {
        match self {
            Sex::Male => "мужской".to_string(),
            Sex::Female => "женский".to_string(),
            Sex::NotDeclaredYet => "не опеределен".to_string(),
        }
    }
}

impl Default for Sex {
    fn default() -> Self {
        Sex::NotDeclaredYet
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumIter)]
pub enum DataType {
    Password {
        password: String,
    },

    Card {
        num: String,
        cvv: u16,
        bank: String,
    },

    Passport {
        fsl: String,
        date: String,
        sex: Sex,
        serial: u16,
        num: u32,
    },

    Document {
        text: String,
    },

    WifiConfig {
        name: String,
        password: String,
    },

    Token {
        token: String,
        from: String,
    },
}

impl DataType {
    pub fn to_string(&self) -> String {
        match self {
            DataType::Password { password } => {
                format!("Пароль: {}", password)
            }
            DataType::Card { num, cvv, bank } => {
                format!("Номер: {},\n CVV: {},\n, Банк: {}", num, cvv, bank)
            }
            DataType::Document { text } => {
                format!("Содержимое : {}", text)
            }
            DataType::Token { token, from } => {
                format!("От: {},\n Токен: {}", from, token)
            }
            DataType::WifiConfig { name, password } => {
                format!("Название сети: {},\n Пароль: {}", name, password)
            }
            DataType::Passport {
                fsl,
                date,
                sex,
                serial,
                num,
            } => {
                format!(
                    "ФИО: {},\n Дата рождения: {},\n Пол: {},\n Серия: {},\n Номер: {}",
                    fsl,
                    date,
                    sex.to_string(),
                    serial,
                    num
                )
            }
        }
    }

    pub fn name(&self) -> String {
        match self {
            DataType::Card { .. } => {
                return "Банковская карта (номер, CVV, банк)".to_string();
            }
            DataType::Token { .. } => return "Токен (токен, от чего)".to_string(),
            DataType::Document { .. } => return "Документ (текст)".to_string(),
            DataType::Passport { .. } => {
                return "Пасспорт (ФИО, дата рождения, пол, серия, номер)".to_string();
            }
            DataType::Password { .. } => return "Пароль (пароль)".to_string(),
            DataType::WifiConfig { .. } => {
                return "Wifi сеть (ip/название, пароль)".to_string();
            }
        }
    }

    pub fn formal_name(&self) -> String {
        match self {
            DataType::Card { .. } => "card".to_string(),
            DataType::Token { .. } => "token".to_string(),
            DataType::Document { .. } => "document".to_string(),
            DataType::Passport { .. } => "passport".to_string(),
            DataType::Password { .. } => "password".to_string(),
            DataType::WifiConfig { .. } => "wificonfig".to_string(),
        }
    }
}

fn new_card() -> DataType {
    DataType::Card {
        num: "".to_string(),
        cvv: 0,
        bank: "".to_string(),
    }
}

fn new_token() -> DataType {
    DataType::Token {
        token: "".to_string(),
        from: "".to_string(),
    }
}

fn new_doc() -> DataType {
    DataType::Document {
        text: "".to_string(),
    }
}

fn new_passport() -> DataType {
    DataType::Passport {
        fsl: "".to_string(),
        date: "".to_string(),
        sex: Sex::NotDeclaredYet,
        serial: 0,
        num: 0,
    }
}

fn new_wificonfig() -> DataType {
    DataType::WifiConfig {
        name: "".to_string(),
        password: "".to_string(),
    }
}

fn new_password() -> DataType {
    DataType::Password {
        password: "".to_string(),
    }
}

pub struct UserData {
    pub id: i64,
    pub data: Vec<u8>,
    pub data_type: DataType,
    pub nonce: [u8; 24],
    pub notice: Vec<u8>,
    pub name: Vec<u8>,
}

impl UserData {
    pub fn new(
        id: i64,
        data: Vec<u8>,
        data_type: String,
        nonce: [u8; 24],
        notice: Vec<u8>,
        name: Vec<u8>,
    ) -> Self {
        let data_typ: DataType;
        match data_type.as_str() {
            "password" => data_typ = new_password(),
            "passport" => data_typ = new_passport(),
            "document" => data_typ = new_doc(),
            "wificonfig" => data_typ = new_wificonfig(),
            "card" => data_typ = new_card(),
            "token" => data_typ = new_token(),
            _ => unreachable!("ОШИБКА ТИПА ДАННЫХ В БД!"),
        }
        UserData {
            id,
            data,
            data_type: data_typ,
            nonce,
            notice,
            name,
        }
    }
}

pub struct DataAndMeta {
    pub data: DataType,
    pub name: String,
    pub notice: String,
}

impl DataAndMeta {
    pub fn new(data: DataType, name: String, notice: String) -> Self {
        DataAndMeta { data, name, notice }
    }
}
