mod cli;
mod db;
mod os_work;

fn main() {
    cli::hi();

    let (response, path, os) = os_work::verify_data();

    if !response {
        cli::check_error(&os);
        if let Err(err) = os_work::init_dir(&path) {
            cli::throw_err(err.to_string());
        }
        cli::success_init_dir(&path);
    } else {
        cli::check_success();
    }

    if os_work::check_exists(&path, "passhive.db") {
        unreachable!("Неожиданно...");
    } else {
        if let Err(err) = db::init_db(&path) {
            cli::throw_err(err.to_string());
        }
        cli::success_init_db();
    }
}
