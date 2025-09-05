use std::{env, fs::create_dir_all, path::PathBuf, process::Command};

fn get_data_dir() -> (PathBuf, String) {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    return (
        PathBuf::from(home).join(".local/share/PassHiveCLI"),
        "Linux/FreeBSD".to_string(),
    );

    #[cfg(target_os = "macos")]
    return (
        PathBuf::from(home).join("Library/Application Support/PassHiveCLI"),
        "MacOS".to_string(),
    );

    #[cfg(target_os = "windows")]
    return (
        PathBuf::from(env::var("APPDATA").unwrap_or_else(|_| ".".into())).join("PassHiveCLI"),
        "Windows".to_string(),
    );
}

pub fn verify_data() -> (bool, PathBuf, String) {
    let (path, os) = get_data_dir();
    if path.exists() && path.is_dir() {
        return (true, path, os);
    }
    (false, path, os)
}

pub fn init_dir(path: &PathBuf) -> Result<(), std::io::Error> {
    create_dir_all(path.as_path())?;
    Ok(())
}

pub fn check_exists(path: &PathBuf, name: &str) -> bool {
    let file_path = path.join(name);
    file_path.exists() && file_path.is_file()
}

pub fn check_rust_installed() -> (bool, bool) {
    let cargo = Command::new("cargo").arg("--version").output().is_ok();
    let rustc = Command::new("rustc").arg("--version").output().is_ok();
    (cargo, rustc)
}

pub fn install_rust() {
    #[cfg(target_os = "linux")]
    {
        let status = Command::new("sh")
            .arg("-c")
            .arg("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y")
            .status()
            .expect("Не удалось запустить установку Rust (Linux)");

        if status.success() {
            println!("Rust успешно установлен на Linux!");
        } else {
            eprintln!("Установка Rust завершилась с ошибкой.");
        }
    }

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("cmd")
            .arg("/C")
            .arg("curl -sSf https://sh.rustup.rs | sh -s -- -y")
            .status()
            .expect("Не удалось запустить установку Rust (Windows)");

        if status.success() {
            println!("Rust успешно установлен на Windows!");
        } else {
            eprintln!("Установка Rust завершилась с ошибкой.");
        }
    }

    #[cfg(target_os = "freebsd")]
    {
        let _ = Command::new("pkg")
            .arg("install")
            .arg("-y")
            .arg("curl")
            .status();

        let status = Command::new("sh")
            .arg("-c")
            .arg("curl https://sh.rustup.rs -sSf | sh -s -- -y")
            .status()
            .expect("Не удалось запустить установку Rust (FreeBSD)");

        if status.success() {
            println!("Rust успешно установлен на FreeBSD!");
        } else {
            eprintln!("Установка Rust завершилась с ошибкой.");
        }
    }
}
