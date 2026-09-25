pub const ALLOWED_TYPES: [&str; 7] = [
    "image/png",
    "image/svg+xml",
    "image/x-icon",
    "image/vnd.microsoft.icon",
    "image/webp",
    "image/jpeg",
    "image/gif",
];

pub fn sniff(bytes: &[u8], declared: Option<&str>) -> Option<String> {
    let declared = declared
        .map(|value| {
            value
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
        })
        .filter(|value| ALLOWED_TYPES.contains(&value.as_str()));
    let seen = kind_of(bytes)?;
    match declared {
        Some(declared) if same_family(&declared, seen) => Some(declared),
        Some(_) => None,
        None => Some(seen.to_string()),
    }
}

pub fn extension_of(content_type: &str) -> &'static str {
    match content_type {
        "image/png" => "png",
        "image/svg+xml" => "svg",
        "image/webp" => "webp",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        _ => "ico",
    }
}

fn kind_of(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("image/png");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    if bytes.starts_with(b"\xff\xd8\xff") {
        return Some("image/jpeg");
    }
    if bytes.starts_with(b"\x00\x00\x01\x00") {
        return Some("image/x-icon");
    }
    if bytes.len() > 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]).to_ascii_lowercase();
    let head = head.trim_start();
    (head.starts_with("<svg") || (head.starts_with("<?xml") && head.contains("<svg")))
        .then_some("image/svg+xml")
}

fn same_family(declared: &str, seen: &str) -> bool {
    let icon = ["image/x-icon", "image/vnd.microsoft.icon"];
    declared == seen || (icon.contains(&declared) && icon.contains(&seen))
}
