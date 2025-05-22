use std::{env, path::PathBuf};

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

pub fn verify_data() -> (bool, Option<PathBuf>, Option<String>) {
    let (path, os) = get_data_dir();
    if path.exists() && path.is_dir() {
        return (true, None, None);
    }
    (false, Some(path), Some(os))
}

pub fn init_dir(path: &PathBuf) {}
