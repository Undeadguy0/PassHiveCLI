use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct UserMeta {
    id: i64,
    login: String,
    created_at: String,
    hash: String,
    salt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Sex {
    Male,
    Female,
}
impl Sex {
    fn to_string(&self) -> String {
        match self {
            Sex::Male => "мужской".to_string(),
            Sex::Female => "женский".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
        num: u16,
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
    fn to_string(&self) -> String {
        match self {
            DataType::Password { password } => {
                format!("Пароль: {}\n", password)
            }
            DataType::Card { num, cvv, bank } => {
                format!("Номер: {},\n CVV: {},\n, Банк: {}\n", num, cvv, bank)
            }
            DataType::Document { text } => {
                format!("Содержимое : {}\n", text)
            }
            DataType::Token { token, from } => {
                format!("От: {},\n Токен: {}\n", from, token)
            }
            DataType::WifiConfig { name, password } => {
                format!("Название сети: {},\n Пароль: {}\n", name, password)
            }
            DataType::Passport {
                fsl,
                date,
                sex,
                serial,
                num,
            } => {
                format!(
                    "ФИО: {},\n Дата рождения: {},\n Пол: {},\n Серия: {},\n Номер: {}\n",
                    fsl,
                    date,
                    sex.to_string(),
                    serial,
                    num
                )
            }
        }
    }
}
