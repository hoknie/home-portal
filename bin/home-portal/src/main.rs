use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    home_portal::start().await
}
