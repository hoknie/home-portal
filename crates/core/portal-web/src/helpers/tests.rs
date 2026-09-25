use super::looks_like_asset;

#[test]
fn client_side_routes_are_not_assets_even_with_dots() {
    for route in [
        "",
        "services/",
        "services/edit/",
        "nas.local",
        "services/1.2.3.4",
        "settings/network",
    ] {
        assert!(!looks_like_asset(route), "{route}");
    }
}

#[test]
fn files_with_known_extensions_are_assets() {
    for asset in [
        "_next/static/chunks/main.js",
        "favicon.ico",
        "index.html",
        "fonts/inter.woff2",
        "manifest.webmanifest",
    ] {
        assert!(looks_like_asset(asset), "{asset}");
    }
}
