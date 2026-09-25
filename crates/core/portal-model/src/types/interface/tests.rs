use super::Language;

const FALLBACK: Language = Language::En;

#[test]
fn the_best_quality_wins_and_a_region_matches_its_language() {
    assert_eq!(
        Language::negotiate(Some("es-ES,es;q=0.9,en;q=0.8"), FALLBACK),
        Language::Es
    );
    assert_eq!(
        Language::negotiate(Some("en;q=0.5, ru-RU;q=0.7"), FALLBACK),
        Language::Ru
    );
}

#[test]
fn an_unsupported_list_with_a_wildcard_gives_the_fallback() {
    assert_eq!(
        Language::negotiate(Some("de, *;q=0.1"), Language::Ru),
        Language::Ru
    );
}

#[test]
fn a_zero_quality_excludes_a_language() {
    assert_eq!(
        Language::negotiate(Some("ru;q=0, en"), Language::Es),
        Language::En
    );
}

#[test]
fn an_empty_or_garbage_header_gives_the_fallback() {
    for header in [None, Some(""), Some(";;,,q=x"), Some("de-DE")] {
        assert_eq!(
            Language::negotiate(header, Language::Es),
            Language::Es,
            "{header:?}"
        );
    }
}

#[test]
fn matching_ignores_case() {
    assert_eq!(Language::negotiate(Some("RU"), FALLBACK), Language::Ru);
    assert_eq!(Language::parse(" Es "), Some(Language::Es));
    assert_eq!(Language::parse("de"), None);
}

#[test]
fn every_language_has_a_code_and_its_own_name() {
    let codes: Vec<&str> = Language::ALL
        .iter()
        .map(|language| language.code())
        .collect();
    assert_eq!(codes, ["en", "ru", "es"]);
    assert_eq!(Language::Ru.native_name(), "Русский");
}
