use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use strum::IntoEnumIterator;
use unicode_width::UnicodeWidthStr;

use super::db::models;
use crate::ShowableData;
use crate::db::models::{DataAndMeta, DataType};
use colored::{ColoredString, Colorize, Style};
use rpassword::read_password;
use std::cmp::{max, min};
use std::io::{Write, stdin, stdout};
use std::{collections::BTreeMap, path::PathBuf};
use std::{num, usize};

fn titlemaker(len: usize, title: &String, chars: &String) -> String {
    if title.len() >= len {
        return title.clone(); // или можно добавить декора минимальной длины
    }

    let decorator = (len - title.len()) / 2 + 1;
    chars.repeat(decorator) + title + &chars.repeat(decorator)
}

trait ColoredAsStr {
    fn colored_repeat(&self, times: usize) -> ColoredString;
}

trait StrAsColored {
    fn colorize(&self, rgb: &(u8, u8, u8)) -> ColoredString;
}

impl StrAsColored for String {
    fn colorize(&self, rgb: &(u8, u8, u8)) -> ColoredString {
        self.truecolor(rgb.0, rgb.1, rgb.2)
    }
}
impl ColoredAsStr for ColoredString {
    fn colored_repeat(&self, times: usize) -> ColoredString {
        let fgcolor = self.fgcolor;
        let bgcolor = self.bgcolor;
        let style = self.style;

        let mut total = self.input.repeat(times).white();

        if let Some(f) = fgcolor {
            total.fgcolor = Some(f);
        }
        if let Some(b) = bgcolor {
            total.bgcolor = Some(b);
        }
        total.style = style;
        total
    }
}

pub struct TableStyle {
    horizontal_frame: ColoredString,
    horizontal_inner: ColoredString,
    vertical_frame: ColoredString,
    vertical_inner: ColoredString,
    split: ColoredString,
    header_color_rgb: (u8, u8, u8),
    text_color_rgb: (u8, u8, u8),
}

impl TableStyle {
    pub fn new(
        horizontal_frame: ColoredString,
        horizontal_inner: ColoredString,
        vertical_frame: ColoredString,
        vertical_inner: ColoredString,
        split: ColoredString,
        header_color_rgb: (u8, u8, u8),
        text_color_rgb: (u8, u8, u8),
    ) -> Self {
        TableStyle {
            header_color_rgb,
            text_color_rgb,
            horizontal_frame,
            horizontal_inner,
            vertical_frame,
            vertical_inner,
            split,
        }
    }
}

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
        "Перейти в расширенный режим отображения -".truecolor(246, 196, 32),
        "CTRL + T".bold()
    );
    println!(
        "{} {}",
        "Выйти из приложения -".truecolor(246, 196, 32),
        "CTRL + E".bold()
    );
    enable_raw_mode().unwrap();
}

pub fn get_new_row_data() -> Result<DataAndMeta, String> {
    disable_raw_mode().unwrap();

    let mut input = String::new();

    print!(
        "{}",
        "Введите название новой записи (опционально):".truecolor(246, 196, 32)
    );
    stdout().flush().unwrap();
    stdin()
        .read_line(&mut input)
        .map_err(|_| "Ошибка считывания строки!".to_string())?;
    let name = input.trim().to_string();
    input.clear();

    print!(
        "\n{}",
        "Введите пояснение к записи (опционально):".truecolor(246, 196, 32)
    );
    stdout().flush().unwrap();
    stdin()
        .read_line(&mut input)
        .map_err(|_| "Ошибка считывания строки!".to_string())?;
    let notice = input.trim().to_string();
    input.clear();

    let mut data = DataType::Password {
        password: "8".to_string(),
    };
    let all_data_types: Vec<DataType> = DataType::iter().collect();
    println!(
        "\n{}",
        "########## Типы записей ##########".truecolor(246, 196, 32)
    );
    for (idx, dt) in all_data_types.iter().enumerate() {
        println!("{}: {}", (idx + 1).to_string().bold(), dt.name());
    }
    print!(
        "{}",
        "\nВыберите тип записи (номер): ".truecolor(246, 196, 32)
    );
    stdout().flush().unwrap();

    let ind_of_type: usize;
    loop {
        input.clear();
        stdin()
            .read_line(&mut input)
            .map_err(|_| "Ошибка считывания строки!".to_string())?;
        match input.trim().parse::<usize>() {
            Err(_) => {
                println!(
                    "\n{}",
                    "Ошибка номера типа данных! Попробуйте еще раз."
                        .purple()
                        .bold()
                );
                continue;
            }
            Ok(ind) if ind == 0 || ind > all_data_types.len() => {
                println!(
                    "\n{}",
                    "Ошибка номера типа данных! Попробуйте еще раз."
                        .purple()
                        .bold()
                );
                continue;
            }
            Ok(ind) => {
                ind_of_type = ind - 1;
                break;
            }
        }
    }

    match &all_data_types[ind_of_type] {
        DataType::Card { .. } => {
            let card_num: String;
            loop {
                print!(
                    "\n{} ",
                    "Введите номер карты (16 цифр):".truecolor(246, 196, 32)
                );
                stdout().flush().unwrap();
                input.clear();
                stdin()
                    .read_line(&mut input)
                    .map_err(|_| "Ошибка считывания строки!".to_string())?;
                let trimmed = input.trim();
                if trimmed.len() != 16 {
                    println!("\n{}", "В номере карты 16 цифр!".purple().bold());
                    continue;
                }
                if !trimmed.chars().all(|c| c.is_ascii_digit()) {
                    println!(
                        "\n{}",
                        "Номер карты должен содержать только цифры!".purple().bold()
                    );
                    continue;
                }
                card_num = trimmed.to_string();
                break;
            }

            let card_cvv: u16;
            loop {
                print!(
                    "\n{} ",
                    "Введите CVV карты (4 цифры):".truecolor(246, 196, 32)
                );
                stdout().flush().unwrap();
                input.clear();
                stdin()
                    .read_line(&mut input)
                    .map_err(|_| "Ошибка считывания строки!".to_string())?;
                let trimmed = input.trim();
                if trimmed.len() != 4 {
                    println!("\n{}", "В CVV должно быть 4 цифры!".purple().bold());
                    continue;
                }
                if !trimmed.chars().all(|c| c.is_ascii_digit()) {
                    println!("\n{}", "CVV должен содержать только цифры!".purple().bold());
                    continue;
                }
                card_cvv = trimmed
                    .parse::<u16>()
                    .map_err(|_| "Ошибка при парсинге CVV!".to_string())?;
                break;
            }

            print!(
                "\n{}",
                "Введите название банка (опционально): ".truecolor(246, 196, 32)
            );
            stdout().flush().unwrap();
            input.clear();
            stdin()
                .read_line(&mut input)
                .map_err(|_| "Ошибка считывания строки!".to_string())?;
            let card_bank = input.trim().to_string();

            let data = DataType::Card {
                num: card_num,
                cvv: card_cvv,
                bank: card_bank,
            };
        }
        DataType::Token { .. } => {
            print!(
                "\n{}",
                "Введите название сервиса (например, GitHub): ".truecolor(246, 196, 32)
            );
            stdout().flush().unwrap();
            input.clear();
            stdin()
                .read_line(&mut input)
                .map_err(|_| "Ошибка считывания строки!".to_string())?;
            let from = input.trim().to_string();
            print!("\n{}", "Введите токен: ".truecolor(246, 196, 32));
            stdout().flush().unwrap();
            input.clear();
            stdin()
                .read_line(&mut input)
                .map_err(|_| "Ошибка считывания строки!".to_string())?;
            let token = input.trim().to_string();

            let result = DataType::Token { token, from };
            data = result;
        }
        DataType::Password { .. } => {
            print!("\n{}", "Введите пароль: ".truecolor(246, 196, 32));
            stdout().flush().unwrap();
            let password =
                rpassword::read_password().map_err(|_| "Ошибка считывания пароля!".to_string())?;

            print!("\n{}", "Повторите пароль: ".truecolor(246, 196, 32));
            stdout().flush().unwrap();
            let password_confirm =
                rpassword::read_password().map_err(|_| "Ошибка считывания пароля!".to_string())?;

            if password != password_confirm {
                println!("\n{}", "Пароли не совпадают!".purple().bold());
                enable_raw_mode().unwrap();
                return Err("Пароли не совпали".to_string());
            }

            let result = DataType::Password { password };
            data = result;
        }
        DataType::WifiConfig { .. } => {
            print!("\n{}", "Введите имя Wi-Fi сети: ".truecolor(246, 196, 32));
            stdout().flush().unwrap();
            input.clear();
            stdin()
                .read_line(&mut input)
                .map_err(|_| "Ошибка считывания строки!".to_string())?;
            let name = input.trim().to_string();

            print!("\n{}", "Введите пароль Wi-Fi: ".truecolor(246, 196, 32));
            stdout().flush().unwrap();
            input.clear();
            stdin()
                .read_line(&mut input)
                .map_err(|_| "Ошибка считывания строки!".to_string())?;
            let password = input.trim().to_string();

            let result = DataType::WifiConfig { name, password };
            data = result;
        }
        DataType::Document { .. } => {
            println!(
                "\n{}",
                "Введите содержимое документа (завершите ввод пустой строкой):"
                    .truecolor(246, 196, 32)
            );
            let mut text_lines = Vec::new();
            loop {
                input.clear();
                stdin()
                    .read_line(&mut input)
                    .map_err(|_| "Ошибка считывания строки!".to_string())?;
                let line = input.trim_end();
                if line.is_empty() {
                    break;
                }
                text_lines.push(line.to_string());
            }
            let text = text_lines.join("\n");

            let result = DataType::Document { text };
            data = result;
        }
        DataType::Passport { .. } => {
            print!("\n{}", "Введите ФИО: ".truecolor(246, 196, 32));
            stdout().flush().unwrap();
            input.clear();
            stdin()
                .read_line(&mut input)
                .map_err(|_| "Ошибка считывания строки!".to_string())?;
            let fsl = input.trim().to_string();
            print!(
                "\n{}",
                "Введите дату рождения (ДД.ММ.ГГГГ): ".truecolor(246, 196, 32)
            );
            stdout().flush().unwrap();
            input.clear();
            stdin()
                .read_line(&mut input)
                .map_err(|_| "Ошибка считывания строки!".to_string())?;
            let date = input.trim().to_string();

            print!("\n{}", "Введите пол (м/ж): ".truecolor(246, 196, 32));
            stdout().flush().unwrap();
            input.clear();
            stdin()
                .read_line(&mut input)
                .map_err(|_| "Ошибка считывания строки!".to_string())?;
            let sex = match input.trim().to_lowercase().as_str() {
                "м" | "male" => models::Sex::Male,
                "ж" | "female" => models::Sex::Female,
                _ => {
                    println!(
                        "\n{}",
                        "Неверный ввод пола, по умолчанию мужской.".purple().bold()
                    );
                    models::Sex::Male
                }
            };

            let serial: u16;
            loop {
                print!(
                    "\n{}",
                    "Введите серию паспорта (4 цифры): ".truecolor(246, 196, 32)
                );
                stdout().flush().unwrap();
                input.clear();
                stdin()
                    .read_line(&mut input)
                    .map_err(|_| "Ошибка считывания строки!".to_string())?;
                let trimmed = input.trim();
                if trimmed.len() != 4 || !trimmed.chars().all(|c| c.is_ascii_digit()) {
                    println!("\n{}", "Серия должна быть из 4 цифр!".purple().bold());
                    continue;
                }
                serial = trimmed
                    .parse::<u16>()
                    .map_err(|_| "Ошибка парсинга серии!".to_string())?;
                break;
            }

            let num: u16;
            loop {
                print!(
                    "\n{}",
                    "Введите номер паспорта (6 цифр): ".truecolor(246, 196, 32)
                );
                stdout().flush().unwrap();
                input.clear();
                stdin()
                    .read_line(&mut input)
                    .map_err(|_| "Ошибка считывания строки!".to_string())?;
                let trimmed = input.trim();
                if trimmed.len() != 6 || !trimmed.chars().all(|c| c.is_ascii_digit()) {
                    println!("\n{}", "Номер должен быть из 6 цифр!".purple().bold());
                    continue;
                }
                num = trimmed
                    .parse::<u16>()
                    .map_err(|_| "Ошибка парсинга номера!".to_string())?;
                break;
            }

            let result = DataType::Passport {
                fsl,
                date,
                sex,
                serial,
                num,
            };
            let data = result;
        }
    }
    enable_raw_mode().unwrap();
    return Ok(DataAndMeta::new(data, name, notice));
}

pub fn show_data_extended(data: &BTreeMap<String, Vec<ShowableData>>, style: &TableStyle) {
    disable_raw_mode().expect("Ошибка выходы из сырого режима!");
    let mut len = 0usize;
    let mut size = 0usize;
    let mut counter = 1usize;

    for section in data {
        if let Some(first) = section.1.first() {
            len = max(len, first.data.name().len());
        }

        for item in section.1 {
            size += 1;
            let max_line_len = item
                .data
                .to_string()
                .split('\n')
                .map(|line| line.len())
                .max()
                .unwrap_or(0);

            len = max(len, max_line_len);
        }
    }

    let num_buf = size.to_string().len();
    println!(
        "{}",
        style.horizontal_frame.colored_repeat(len + num_buf + 7)
    );

    for section in data {
        match section.1.first() {
            Some(d) => {
                let title_raw = d.data.name();
                let title_colored = title_raw.colorize(&style.header_color_rgb);
                let visible_width = UnicodeWidthStr::width(title_raw.as_str());
                let total_width = len + num_buf + 4;
                let padding = total_width.saturating_sub(visible_width - 1);

                println!(
                    "{} {} {}",
                    style.vertical_frame,
                    style.horizontal_inner.colored_repeat(total_width),
                    style.vertical_frame
                );

                println!(
                    "{} {}{}{}",
                    style.vertical_frame,
                    title_colored,
                    " ".repeat(padding),
                    style.vertical_frame
                );

                println!(
                    "{} {} {}",
                    style.vertical_frame,
                    style.horizontal_inner.colored_repeat(total_width),
                    style.vertical_frame
                );
            }
            None => {}
        }
        if section.1.len() != 0 {
            for data in section.1 {
                println!(
                    "{} {} {} Название:{}{}{}",
                    style.vertical_frame,
                    counter,
                    style.vertical_inner,
                    data.name.colorize(&style.text_color_rgb),
                    " ".to_string().repeat(len - data.name.len() - 7),
                    style.vertical_frame,
                );

                println!(
                    "{} {} {} Заметка:{}{}{}",
                    style.vertical_frame,
                    " ".repeat(num_buf),
                    style.vertical_inner,
                    data.notice.colorize(&style.text_color_rgb),
                    " ".to_string().repeat(len - data.notice.len() - 6),
                    style.vertical_frame,
                );

                for line in data.data.to_string().split("\n") {
                    let visible_width = UnicodeWidthStr::width(line);
                    let pad = len.saturating_sub(visible_width);

                    println!(
                        "{} {} {} {}{}{}",
                        style.vertical_frame,
                        " ".repeat(num_buf),
                        style.vertical_inner,
                        line.to_string().colorize(&style.text_color_rgb),
                        " ".repeat(pad + 2),
                        style.vertical_frame,
                    );
                }

                println!(
                    "{}{}{}",
                    style.vertical_frame,
                    style.split.colored_repeat(len + num_buf + 6),
                    style.vertical_frame,
                );

                counter += 1;
            }
        }
    }
    println!(
        "{}",
        style.horizontal_frame.colored_repeat(len + num_buf + 7)
    );
    enable_raw_mode().expect("Ошибка входа в сырой режим!");
}
