use super::{content_type_of, looks_like_asset};

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

#[test]
fn a_known_extension_has_its_content_type_whatever_its_case() {
    assert_eq!(
        content_type_of("en/index.HTML"),
        Some("text/html; charset=utf-8")
    );
    assert_eq!(
        content_type_of("_next/static/chunk.js"),
        Some("text/javascript")
    );
    assert_eq!(content_type_of("en/services.v2/"), None);
    assert_eq!(content_type_of("README"), None);
}
