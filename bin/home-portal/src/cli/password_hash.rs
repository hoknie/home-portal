use std::io::{self, BufRead, IsTerminal};
use std::process::ExitCode;

use portal_auth::hash_password;

use super::failure::fail;

pub const PROMPT: &str = "Password: ";

pub fn password_hash() -> ExitCode {
    let password = match read_password() {
        Ok(password) => password,
        Err(error) => return fail(format!("cannot read the password: {error}")),
    };
    if password.is_empty() {
        return fail("the password must not be empty");
    }
    match hash_password(&password) {
        Ok(hash) => {
            println!("{hash}");
            ExitCode::SUCCESS
        }
        Err(error) => fail(format!("cannot hash the password: {error}")),
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
