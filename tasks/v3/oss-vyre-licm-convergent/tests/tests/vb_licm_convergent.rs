// fail_to_pass: `is_pure` must return `false` for convergent subgroup ops.
// The buggy `is_pure` classifies these as pure (true), making them eligible for
// LICM hoisting.  The fix requires moving these to the non-pure arm.

use oss_vyre_licm_convergent::{is_pure, OpKind};

#[test]
fn reduce_is_not_pure() {
    assert!(!is_pure(&OpKind::SubgroupReduce),
        "SubgroupReduce is convergent — must NOT be hoistable");
}

#[test]
fn ballot_is_not_pure() {
    assert!(!is_pure(&OpKind::SubgroupBallot),
        "SubgroupBallot is convergent — must NOT be hoistable");
}

#[test]
fn shuffle_is_not_pure() {
    assert!(!is_pure(&OpKind::SubgroupShuffle),
        "SubgroupShuffle is convergent — must NOT be hoistable");
}

#[test]
fn broadcast_is_not_pure() {
    assert!(!is_pure(&OpKind::SubgroupBroadcast),
        "SubgroupBroadcast is convergent — must NOT be hoistable");
}
