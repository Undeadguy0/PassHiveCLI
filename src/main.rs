mod cli;
mod os_work;

fn main() {
    cli::hi();

    let (response, path, os) = os_work::verify_data();
    if response {
        cli::check_success();
    } else {
        let path = path.unwrap().clone();
        cli::check_error(&os.unwrap());
        os_work::init_dir(&path);
        cli::success_init_dir(&path);
        cli::init_db();
    }
}
