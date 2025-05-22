use colored::*;
use std::{
    io::{Write, stdout},
    path::PathBuf,
};

const main_yellow: (u8, u8, u8) = (246, 196, 32);

pub fn hi() {
    println!("{}", "🐝🐝🐝 Приветствую в PassHiveCLI! 🐝🐝🐝".green());
}

pub fn check_success() {
    print!(
        "{}",
        "Подключение к рабочей директории ".truecolor(246, 196, 32)
    );
    print!("{}", "успешно!".green().bold());
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
    print!("{}", "создана успешно!".green().bold());
}

pub fn init_db() {
    println!("{}", "Инициализурую БД, бзз".truecolor(246, 196, 32));
}

pub fn throw_err(msg: String) {
    println!("{}", msg.bold().red());
}
