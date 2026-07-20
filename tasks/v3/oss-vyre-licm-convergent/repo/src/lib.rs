//! Loop-invariant code motion (LICM) "is-pure" gate for an internal IR.
//!
//! Bug: the `is_pure` predicate classifies convergent subgroup collectives
//! (SubgroupBallot, SubgroupShuffle, SubgroupBroadcast, SubgroupReduce) as
//! "pure", meaning safe to hoist out of a loop.  These are CONVERGENT ops:
//! their result depends on which lanes are active at the program point
//! *inside* the loop.  Hoisting one changes its execution count (N → 1) and
//! therefore the set of participating lanes, producing a different result
//! under the same inputs — a miscompile.
//!
//! The fix moves the four convergent ops to return `false`.
//! (The real bug was vyre-lower's descriptor LICM pass, commit 165d952c.)

/// Kinds of operations in the internal IR.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpKind {
    Arithmetic,
    SubgroupReduce,
    SubgroupBallot,
    SubgroupShuffle,
    SubgroupBroadcast,
    SubgroupLocalId,
    Store,
}

/// Return `true` if an op kind is "pure" (safe to hoist out of a loop).
///
/// **BUGGY VERSION**: convergent subgroup collectives return `true`
/// even though their result depends on the set of active lanes at the
/// program point (which changes when hoisted).
pub fn is_pure(kind: &OpKind) -> bool {
    match kind {
        // Arithmetic ops: always safe to hoist.
        OpKind::Arithmetic => true,

        // ── BUG ──────────────────────────────────────────────
        // Convergent subgroup collectives classified as pure.
        OpKind::SubgroupReduce
        | OpKind::SubgroupBallot
        | OpKind::SubgroupShuffle
        | OpKind::SubgroupBroadcast
        | OpKind::SubgroupLocalId => true,

        // Side-effecting: never hoist.
        OpKind::Store => false,
    }
}
