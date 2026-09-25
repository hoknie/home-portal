use std::net::IpAddr;

pub const ECHO_REQUEST_V4: u8 = 8;
pub const ECHO_REPLY_V4: u8 = 0;
pub const ECHO_REQUEST_V6: u8 = 128;
pub const ECHO_REPLY_V6: u8 = 129;
pub const ECHO_HEADER_LENGTH: usize = 8;
pub const TOKEN_LENGTH: usize = 8;

pub fn echo_request(target: IpAddr, sequence: u16, token: [u8; TOKEN_LENGTH]) -> Vec<u8> {
    let kind = if target.is_ipv4() {
        ECHO_REQUEST_V4
    } else {
        ECHO_REQUEST_V6
    };
    let mut packet = vec![kind, 0, 0, 0, 0, 0];
    packet.extend_from_slice(&sequence.to_be_bytes());
    packet.extend_from_slice(&token);
    if target.is_ipv4() {
        let sum = checksum(&packet);
        packet[2..4].copy_from_slice(&sum.to_be_bytes());
    }
    packet
}

pub fn is_echo_reply(received: &[u8], sequence: u16, token: [u8; TOKEN_LENGTH]) -> bool {
    let message = strip_ip_header(received);
    if message.len() < ECHO_HEADER_LENGTH + TOKEN_LENGTH {
        return false;
    }
    matches!(message[0], ECHO_REPLY_V4 | ECHO_REPLY_V6)
        && message[6..8] == sequence.to_be_bytes()
        && message[ECHO_HEADER_LENGTH..ECHO_HEADER_LENGTH + TOKEN_LENGTH] == token
}

pub fn checksum(bytes: &[u8]) -> u16 {
    let mut sum: u32 = bytes
        .chunks(2)
        .map(|pair| u32::from(u16::from_be_bytes([pair[0], *pair.get(1).unwrap_or(&0)])))
        .sum();
    while sum > 0xffff {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

fn strip_ip_header(received: &[u8]) -> &[u8] {
    match received.first() {
        Some(first) if first >> 4 == 4 => {
            let length = usize::from(first & 0x0f) * 4;
            received.get(length..).unwrap_or(&[])
        }
        _ => received,
    }
}
