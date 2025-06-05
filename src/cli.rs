use colored::*;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use strum::IntoEnumIterator;

use super::db::models;
use crate::ShowableData;
use crate::db::models::DataType;
use colored::Colorize;
use rpassword::read_password;
use std::io::{Write, stdin, stdout};

use std::{collections::BTreeMap, path::PathBuf};

pub struct DataFromUser {
    name: Option<String>,
    notice: Option<String>,
    data_type: models::DataType,
    data: String,
}

impl DataFromUser {
    fn new(
        name: Option<String>,
        notice: Option<String>,
        data_type: models::DataType,
        data: String,
    ) -> Self {
        DataFromUser {
            name,
            notice,
            data_type,
            data,
        }
    }
}

const main_yellow: (u8, u8, u8) = (246, 196, 32);

pub enum AccountManipulation {
    Regist,
    Auth,
}

pub fn hi() {
    println!("{}", "🐝🐝🐝 Приветствую в PassHiveCLI! 🐝🐝🐝".green());
}

pub fn check_success() {
    print!(
        "{}",
        "Подключение к рабочей директории ".truecolor(246, 196, 32)
    );
    print!("{}", "успешно!\n".green().bold());
    stdout().flush().unwrap();
}

pub fn check_error(os: &String) {
    print!(
        "{}",
        "Подключение к рабочей директории ".truecolor(246, 196, 32)
    );
    print!("{}", "не удалось!\n".red().bold());
    stdout().flush().unwrap();

    println!(
        "{}",
        "Инициализирую рабочую директорию, бзз🐝".truecolor(246, 196, 32)
    );
    println!("Ваша ОС - {}", os.italic());
}

pub fn success_init_dir(path: &PathBuf) {
    print!("{}", "Директория ".truecolor(246, 196, 32));
    print!("{} ", path.display());
    print!("{}", "создана успешно!\n".green().bold());
}

pub fn init_db() {
    println!("{}", "Инициализурую БД, бзз\n".truecolor(246, 196, 32));
}

pub fn success_init_db() {
    print!("{}", "База данных ".truecolor(246, 196, 32));
    print!("{}", "создана успешно!\n".green().bold());
}

pub fn throw_err(msg: String) {
    panic!("{}", msg.bold().red());
}

pub fn db_conn_success() {
    print!("{}", "Подключение к БД ".truecolor(246, 196, 32));
    print!("{}", "успешно!\n".green().bold());
}

pub fn registration() -> (String, String) {
    println!("{}", "Регистрация нового аккаунта.".truecolor(246, 196, 32));

    let login = loop {
        let mut input = String::new();
        print!("{}", "Введите логин: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut input).unwrap();

        let input = input.trim();
        if input.is_empty() {
            println!("{}", "Логин не может быть пустым!".red().bold());
        } else if input.len() <= 3 {
            println!("{}", "Логин должен быть больше 3 символов!".red().bold());
        } else if input.contains(' ') {
            println!("{}", "Логин не может содержать пробелы!".red().bold());
        } else {
            break input.to_string();
        }
    };

    let password = loop {
        let mut input = String::new();
        print!(
            "{} ",
            "Вы желаете вводить пароль в открытом виде? (д|y/н|n):".truecolor(246, 196, 32)
        );
        stdout().flush().unwrap();
        stdin().read_line(&mut input).unwrap();

        let secret = match input.trim().to_lowercase().as_str() {
            "д" | "y" => true,
            "н" | "n" => false,
            _ => {
                println!("{}", "Неверный ввод!".red().bold());
                continue;
            }
        };

        let mut buf = String::new();
        print!("{}", "\nВведите пароль: ");
        stdout().flush().unwrap();
        if secret {
            buf = read_password().expect("Ошибка считывания пароля");
        } else {
            stdin().read_line(&mut buf).unwrap();
        }

        print!("{}", "\nПовторите пароль: ");
        stdout().flush().unwrap();
        let mut buf2 = String::new();
        if secret {
            buf2 = read_password().unwrap_or_else(|_| {
                println!("{}", "\nОшибка ввода!".red().bold());
                String::new()
            });
        } else {
            stdin().read_line(&mut buf2).unwrap_or_default();
        }

        if buf.trim() != buf2.trim() {
            println!("{}", "\nПароли не совпадают!".red().bold());
            continue;
        }

        break buf.trim().to_string();
    };

    (login.trim().to_string(), password.trim().to_string())
}

pub fn regist_success(login: &String) {
    print!("{}", "Пользователь ".truecolor(246, 196, 32));
    stdout().flush().unwrap();
    print!("{} ", login);
    stdout().flush().unwrap();
    print!("{}", "зарегистрирован ".truecolor(246, 196, 32));
    print!("{}", "успешно!\n".green().bold());
}

pub fn user_exists_err() {
    println!(
        "{}",
        "Пользователь с таким логином уже существует!"
            .purple()
            .bold()
    );
}

pub fn log_or_reg() -> AccountManipulation {
    let mut input = String::new();

    loop {
        input.clear();
        print!(
            "{}",
            "Вы хотите зарегистрироваться или войти в существующую учетную запись? (р|r/в|s) "
                .truecolor(246, 196, 32)
        );
        stdout().flush().unwrap();

        if let Err(_) = stdin().read_line(&mut input) {
            throw_err(("Ошибка считывания строки").to_string());
        }

        match input.trim() {
            "в" | "s" => return AccountManipulation::Auth,
            "р" | "r" => return AccountManipulation::Regist,
            _ => {
                println!(
                    "{}",
                    "Ошибка ввода типа операции, попробуйте еще раз"
                        .purple()
                        .bold()
                );
                // println!("{input}");
            }
        }
    }
}

pub fn registration_success(login: &String) {
    println!(
        "{} {} {}",
        "Регистрация аккаунта".truecolor(246, 196, 32),
        login.truecolor(246, 196, 32),
        "успешна!".green().bold()
    )
}

pub fn get_auth_data(path: &PathBuf) -> (String, String) {
    let mut input = String::new();

    let login = loop {
        input.clear();

        print!("{}", "Введите логин: ".truecolor(246, 196, 32));
        stdout().flush().unwrap();

        if let Err(_) = stdin().read_line(&mut input) {
            throw_err("Ошибка чтения строки!".to_string());
        }

        if input.trim().len() == 0 {
            println!("{}", "\nЛогин не может быть ПУСТЫМ!".purple().bold());
            continue;
        }

        break input.clone();
    };

    let password = loop {
        input.clear();

        print!("{}", "\nВведите пароль: ".truecolor(246, 196, 32));
        stdout().flush().unwrap();

        match rpassword::read_password() {
            Ok(pass) => {
                if pass.trim().len() == 0 {
                    println!("{}", "\nПароль не может быть пустым!".purple().bold());
                    continue;
                }
                break pass;
            }
            Err(_) => throw_err("Ошибка чтения строки!".to_string()),
        }
    };

    (login.trim().to_string(), password.trim().to_string())
}

pub fn auth_seccess() {
    println!(
        "{} {}",
        "Вход в учетную запись".truecolor(246, 196, 32),
        "успешнeн!".green().bold()
    );
}

pub fn auth_failure() {
    println!("{}", "Неверный пароль или логин!".purple().bold());
}

pub fn show_all_data(data: &BTreeMap<String, Vec<ShowableData>>) {
    let mut counter = 0u32;
    let mut not_empty = false;
    let len = 30usize;

    for entry in data.iter() {
        let header: &str;
        match entry.0.as_str() {
            "password" => header = "пароли",
            "card" => header = "банковские карты",
            "passport" => header = "пасспорты",
            "document" => header = "документы",
            "wificonfig" => header = "Wifi сети",
            "token" => header = "токены",
            _ => unreachable!("Ошибка имени ключа в BTree!!!"),
        }
        if !entry.1.is_empty() {
            not_empty = true;
            let header_len = header.len();
            let decorate = "#".repeat((len - header_len) / 2 + 1);
            println!(
                "{} {} {}.",
                &decorate.truecolor(246, 196, 32),
                &header,
                &decorate.truecolor(246, 196, 32)
            );

            for row in entry.1.iter() {
                let data_in_str = row
                    .data
                    .to_string()
                    .split("\n")
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                println!(
                    "{} {}",
                    (counter.to_string() + ":").truecolor(246, 196, 32),
                    &(row.name).truecolor(246, 196, 32)
                );
                counter += 1;
            }
        }
    }

    if !not_empty {
        println!(
            "{}",
            "########## На данный момент здесь пусто... ########"
                .to_string()
                .truecolor(246, 196, 32)
        );
    }

    println!("")
}

pub fn show_hotkeys() {
    disable_raw_mode().unwrap();
    println!("{}", "Горячие клавиши:".truecolor(246, 196, 32).bold());
    println!(
        "{} {}",
        "Добавить новую запись -".truecolor(246, 196, 32),
        "CTRL + A".bold()
    );
    println!(
        "{} {}",
        "Удалить запись -".truecolor(246, 196, 32),
        "CTRL + D".bold()
    );
    println!(
        "{} {}",
        "Редактировать запись -".truecolor(246, 196, 32),
        "CTRL + U".bold()
    );
    println!(
        "{} {}",
        "Выйти из приложения -".truecolor(246, 196, 32),
        "CTRL + E".bold()
    );
    enable_raw_mode().unwrap();
}

pub fn get_new_row_data() -> Result<DataType, String> {
    disable_raw_mode().unwrap();

    let mut input = String::new();
    print!(
        "{}",
        "Введите название новой записи (опционально):".truecolor(246, 196, 32)
    );
    stdout().flush().unwrap();
    if let Err(_) = stdin().read_line(&mut input) {
        return Err("Ошибка считывания строки!".to_string());
    }

    let name = input.trim().to_string();
    input.clear();

    print!(
        "\n{}",
        "Введите пояснение к записи (опционально): ".truecolor(246, 196, 32)
    );
    stdout().flush().unwrap();
    if let Err(_) = stdin().read_line(&mut input) {
        return Err("Ошибка считывания строки!".to_string());
    }

    let notice = input.trim().to_string();
    input.clear();

    let all_data_types: Vec<DataType> = DataType::iter().collect();
    println!(
        "\n{}",
        "########## Типы записей ##########".truecolor(246, 196, 32)
    );
    for data_type in all_data_types.iter().enumerate() {
        println!(
            "{}: {}",
            (data_type.0 + 1).to_string().bold(),
            data_type.1.name()
        );
    }
    print!(
        "{}: ",
        "Выберите тип записи (номер)".truecolor(246, 196, 32)
    );
    stdout().flush().unwrap();

    let ind_of_type: u8;
    loop {
        input.clear();
        if let Err(_) = stdin().read_line(&mut input) {
            return Err("Ошибка считывания строки!".to_string());
        }
        match input.trim().parse::<u8>() {
            Err(_) => {
                println!(
                    "\n{}",
                    "Ошибка номера типа данных! Попробуйте еще раз"
                        .purple()
                        .bold()
                );
                continue;
            }
            Ok(ind) => {
                if ind as usize > all_data_types.len() {
                    println!(
                        "\n{}",
                        "Ошибка номера типа данных! Попробуйте еще раз"
                            .purple()
                            .bold()
                    );
                    continue;
                } else {
                    ind_of_type = ind;
                    break;
                }
            }
        }
    }

    match &all_data_types[ind_of_type as usize] {
        DataType::Card { num, cvv, bank } => {
            loop {
                print!(
                    "\n{} ",
                    "Введите номер карты (16 цифр):".truecolor(246, 196, 32)
                );
                stdout().flush().unwrap();

                if let Err(_) = stdin().read_line(&mut input) {
                    return Err("Ошибка считывания строки!".to_string());
                }

                let card_num;
                if input.trim().len() != 16 {
                    println!("\n{}", "В номере карты 16 цифр!!!".purple().bold());
                    continue;
                }
                if input.trim().chars().any(|char| !char.is_digit(10)) {
                    println!(
                        "\n{}",
                        "В номере карты должны быть ТОЛЬКО ЦИФРЫ!".purple().bold()
                    );
                    continue;
                }
                card_num = input.trim().to_string();
                break;
            }
            print!(
                "\n{}",
                "Введите CVV карты (4 цифры): ".truecolor(246, 196, 32)
            );
            stdout().flush().unwrap();

            let mut card_cvv: u16;
            loop {
                input.clear();
                if let Err(_) = stdin().read_line(&mut input) {
                    return Err("Ошибка считывания строки!".to_string());
                }
                if input.trim().len() != 4 {
                    println!("\n{}", "В CVV должно быть 4 цифры!!!".purple().bold());
                    continue;
                }
                if input.trim().chars().any(|char| !char.is_digit(10)) {
                    println!("\n{}", "В CVV должны быть ТОЛЬКО ЦИФРЫ!!!".purple().bold());
                    continue;
                }

                match input.trim().parse::<u16>() {
                    Ok(cvv) => {
                        card_cvv = cvv;
                        break;
                    }
                    Err(_) => {
                        println!("\n{}", "Ошибка при вводе числа!!!".purple().bold());
                        continue;
                    }
                }
            }
        }
        DataType::Token { token, from } => {}
        DataType::Password { password } => {}
        DataType::WifiConfig { name, password } => {}
        DataType::Document { text } => {}
        DataType::Passport {
            fsl,
            date,
            sex,
            serial,
            num,
        } => {}
    }

    enable_raw_mode().unwrap();
    return Err("Ошибка считывания строки!".to_string());
}
