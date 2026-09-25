pub const ASSET_EXTENSIONS: [&str; 30] = [
    "js",
    "mjs",
    "css",
    "map",
    "json",
    "txt",
    "xml",
    "wasm",
    "webmanifest",
    "html",
    "htm",
    "png",
    "jpg",
    "jpeg",
    "gif",
    "svg",
    "ico",
    "webp",
    "avif",
    "bmp",
    "woff",
    "woff2",
    "ttf",
    "otf",
    "eot",
    "mp4",
    "webm",
    "mp3",
    "wav",
    "pdf",
];

pub fn looks_like_asset(path: &str) -> bool {
    let last = path
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(path);
    if path.ends_with('/') {
        return false;
    }
    last.rsplit_once('.').is_some_and(|(_, extension)| {
        ASSET_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
    })
}
