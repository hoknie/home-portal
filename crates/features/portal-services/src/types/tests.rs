use super::ServiceEntry;

#[test]
fn normalizing_drops_empty_texts_and_blank_notes_and_keeps_the_rest() {
    let entry = ServiceEntry {
        group: Some(String::new()),
        icon: Some(" ".into()),
        description: Some("a box".into()),
        notes: Some("  \n ".into()),
        ..ServiceEntry::new("media", "Media", "http://10.0.0.5")
    };
    let normalized = entry.normalized();
    assert_eq!(normalized.group, None);
    assert_eq!(normalized.icon.as_deref(), Some(" "));
    assert_eq!(normalized.description.as_deref(), Some("a box"));
    assert_eq!(normalized.notes, None);
    assert_eq!(
        ServiceEntry {
            notes: Some(" kept ".into()),
            ..entry.clone()
        }
        .normalized()
        .notes
        .as_deref(),
        Some(" kept ")
    );
}
