//! Loop-invariant code motion (LICM) purity gate for a small internal IR.

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

/// Return `true` if an op kind is pure (safe to hoist out of a loop).
pub fn is_pure(kind: &OpKind) -> bool {
    match kind {
        OpKind::Arithmetic => true,
        OpKind::SubgroupReduce
        | OpKind::SubgroupBallot
        | OpKind::SubgroupShuffle
        | OpKind::SubgroupBroadcast
        | OpKind::SubgroupLocalId => true,
        OpKind::Store => false,
    }
}
