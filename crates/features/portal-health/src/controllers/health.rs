pub const ALIVE: &str = "ok";

pub async fn health() -> &'static str {
    ALIVE
}
