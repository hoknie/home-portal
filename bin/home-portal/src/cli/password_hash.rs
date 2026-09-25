use std::io::{self, BufRead, IsTerminal};
use std::process::ExitCode;

use portal_auth::hash_password;

pub const PASSWORD_HASH: &str = "password-hash";
pub const PROMPT: &str = "Password: ";

pub fn password_hash() -> ExitCode {
    let password = match read_password() {
        Ok(password) => password,
        Err(error) => {
            eprintln!("home-portal: cannot read the password: {error}");
            return ExitCode::FAILURE;
        }
    };
    if password.is_empty() {
        eprintln!("home-portal: the password must not be empty");
        return ExitCode::FAILURE;
    }
    match hash_password(&password) {
        Ok(hash) => {
            println!("{hash}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("home-portal: cannot hash the password: {error}");
            ExitCode::FAILURE
        }
    }
}

fn read_password() -> io::Result<String> {
    if io::stdin().is_terminal() {
        return rpassword::prompt_password(PROMPT);
    }
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim_end_matches(['\n', '\r']).to_string())
}
