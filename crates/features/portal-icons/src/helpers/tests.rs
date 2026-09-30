use super::{digest_of, extension_of, sniff};

const PNG: &[u8] = b"\x89PNG\r\n\x1a\n rest";
const SVG: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";

#[test]
fn an_image_is_recognised_by_its_bytes() {
    assert_eq!(sniff(PNG, Some("image/png")).as_deref(), Some("image/png"));
    assert_eq!(
        sniff(SVG, Some("image/svg+xml; charset=utf-8")).as_deref(),
        Some("image/svg+xml")
    );
    assert_eq!(sniff(b"GIF89a...", None).as_deref(), Some("image/gif"));
    assert_eq!(
        sniff(b"\x00\x00\x01\x00rest", Some("image/vnd.microsoft.icon")).as_deref(),
        Some("image/vnd.microsoft.icon")
    );
}

#[test]
fn anything_that_is_not_an_image_is_refused() {
    assert!(sniff(b"<!doctype html><html></html>", Some("text/html")).is_none());
    assert!(sniff(b"plain text", None).is_none());
    assert!(sniff(b"%PDF-1.4", Some("application/pdf")).is_none());
}

#[test]
fn bytes_that_disagree_with_the_declared_type_are_refused() {
    assert!(sniff(b"<!doctype html>", Some("image/png")).is_none());
    assert!(sniff(PNG, Some("image/jpeg")).is_none());
}

#[test]
fn every_accepted_type_has_a_file_extension() {
    assert_eq!(extension_of("image/png"), "png");
    assert_eq!(extension_of("image/svg+xml"), "svg");
    assert_eq!(extension_of("image/x-icon"), "ico");
}

#[test]
fn a_digest_is_short_stable_and_follows_the_source() {
    assert_eq!(digest_of("auto"), digest_of("auto"));
    assert_ne!(digest_of("auto"), digest_of("catalog:jellyfin"));
    assert_eq!(digest_of("auto").len(), 32);
}

#[test]
fn an_svg_that_can_run_script_is_refused_like_an_unreadable_image() {
    let active: [&[u8]; 6] = [
        b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script>fetch('/api/users')</script></svg>",
        b"<svg onload=\"fetch('/api/users')\"></svg>",
        b"<svg><a href=\"javascript:alert(1)\"><circle r=\"4\"/></a></svg>",
        b"<svg><foreignObject><iframe src=\"/\"/></foreignObject></svg>",
        b"<svg><image\n onerror = \"x()\" href=\"\"/></svg>",
        b"<?xml version=\"1.0\"?><svg><SCRIPT>x()</SCRIPT></svg>",
    ];
    for bytes in active {
        assert!(
            sniff(bytes, None).is_none(),
            "{}",
            String::from_utf8_lossy(bytes)
        );
    }
    let plain = b"<svg viewBox=\"0 0 24 24\"><path d=\"M1 1h22\" stroke-width=\"2\" fill=\"none\"/><text>online</text></svg>";
    assert_eq!(sniff(plain, None).as_deref(), Some("image/svg+xml"));
}

#[test]
fn an_icon_answer_cannot_run_anything_when_opened_directly() {
    let mut headers = axum::http::HeaderMap::new();
    super::inert_headers(&mut headers, "svg");
    assert_eq!(
        headers["content-security-policy"],
        "default-src 'none'; style-src 'unsafe-inline'; sandbox"
    );
    assert_eq!(headers["x-content-type-options"], "nosniff");
    assert_eq!(
        headers["content-disposition"],
        "inline; filename=\"icon.svg\""
    );
}
