use colored::*;
use rpassword::read_password;
use std::{
    io::{Write, stdin, stdout},
    path::PathBuf,
};

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
    use colored::Colorize;
    use rpassword::read_password;
    use std::io::{Write, stdin, stdout};

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
            _ => println!(
                "{}",
                "Ошибка ввода типа операции, попробуйте еще раз"
                    .purple()
                    .bold()
            ),
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
        "успешна!".green().bold()
    );
}

pub fn auth_failure() {
    println!("{}", "Неверный пароль или логин!".purple().bold());
}
