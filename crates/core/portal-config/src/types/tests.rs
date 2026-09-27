use crate::types::{Revision, Revisioned};

#[test]
fn a_revisioned_value_keeps_its_value_and_revision() {
    let revisioned = Revisioned::new(7, Revision::of(b"settings"));
    assert_eq!(revisioned.value, 7);
    assert_eq!(revisioned.revision, Revision::of(b"settings"));
}

#[test]
fn mapping_a_revisioned_value_keeps_the_revision() {
    let mapped = Revisioned::new(7, Revision::of(b"settings")).map(|value| value.to_string());
    assert_eq!(mapped.value, "7");
    assert_eq!(mapped.revision, Revision::of(b"settings"));
}
