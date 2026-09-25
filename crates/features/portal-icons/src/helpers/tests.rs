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
