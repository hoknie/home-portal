use rand::RngCore;
use rand::rngs::OsRng;

pub const TOKEN_BYTES: usize = 32;

pub fn new_token() -> String {
    let mut bytes = [0u8; TOKEN_BYTES];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
