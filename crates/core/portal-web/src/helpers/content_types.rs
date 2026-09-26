pub const FALLBACK_CONTENT_TYPE: &str = "application/octet-stream";

pub const CONTENT_TYPES: [(&str, &str); 30] = [
    ("js", "text/javascript"),
    ("mjs", "text/javascript"),
    ("css", "text/css"),
    ("map", "application/json"),
    ("json", "application/json"),
    ("txt", "text/plain; charset=utf-8"),
    ("xml", "application/xml"),
    ("wasm", "application/wasm"),
    ("webmanifest", "application/manifest+json"),
    ("html", "text/html; charset=utf-8"),
    ("htm", "text/html; charset=utf-8"),
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("svg", "image/svg+xml"),
    ("ico", "image/x-icon"),
    ("webp", "image/webp"),
    ("avif", "image/avif"),
    ("bmp", "image/bmp"),
    ("woff", "font/woff"),
    ("woff2", "font/woff2"),
    ("ttf", "font/ttf"),
    ("otf", "font/otf"),
    ("eot", "application/vnd.ms-fontobject"),
    ("mp4", "video/mp4"),
    ("webm", "video/webm"),
    ("mp3", "audio/mpeg"),
    ("wav", "audio/wav"),
    ("pdf", "application/pdf"),
];

pub fn content_type_of(path: &str) -> Option<&'static str> {
    let (_, extension) = path.rsplit('/').next()?.rsplit_once('.')?;
    let extension = extension.to_ascii_lowercase();
    CONTENT_TYPES
        .iter()
        .find(|(known, _)| *known == extension)
        .map(|(_, content_type)| *content_type)
}
