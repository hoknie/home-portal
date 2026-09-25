use super::condition_of;
use super::conditions::{CONDITIONS, UNKNOWN_CONDITION};

#[test]
fn every_mapped_code_has_its_own_name() {
    for (code, condition) in CONDITIONS {
        assert_eq!(condition_of(code), condition);
        assert_ne!(condition, UNKNOWN_CONDITION);
    }
}

#[test]
fn a_code_the_portal_does_not_map_is_unknown() {
    assert_eq!(condition_of(7), UNKNOWN_CONDITION);
    assert_eq!(condition_of(-1), UNKNOWN_CONDITION);
    assert_eq!(condition_of(1000), UNKNOWN_CONDITION);
}
