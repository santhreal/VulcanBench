// pass_to_pass: these predicates must keep working before AND after the fix.

use oss_vyre_licm_convergent::{is_pure, OpKind};

#[test]
fn arithmetic_is_pure() {
    assert!(is_pure(&OpKind::Arithmetic),
        "Arithmetic ops must remain hoistable");
}

#[test]
fn local_id_is_pure() {
    assert!(is_pure(&OpKind::SubgroupLocalId),
        "Per-lane constants must remain hoistable");
}

#[test]
fn store_is_not_pure() {
    assert!(!is_pure(&OpKind::Store),
        "Store must never be hoistable");
}
