# LICM `is_pure` incorrectly classifies convergent subgroup collectives as hoistable

The loop-invariant code motion pass uses an `is_pure` predicate to decide whether
an operation can be hoisted out of a loop.  The buggy version classifies four
convergent subgroup collective ops as pure: `SubgroupReduce`, `SubgroupBallot`,
`SubgroupShuffle`, and `SubgroupBroadcast`.

Convergent collectives' results depend on the set of lanes active at the program
point *inside* the loop.  Hoisting one changes its execution count from N to 1
and changes the participation context, producing a different result — a
miscompile.

The `SubgroupLocalId` is a per-lane loop-invariant constant and SHOULD remain
hoistable.

## Expected behavior

`is_pure` must return `false` (not hoistable) for convergent subgroup collectives:

| Op                   | Current | Expected |
|----------------------|---------|----------|
| `SubgroupReduce`     | true    | false    |
| `SubgroupBallot`     | true    | false    |
| `SubgroupShuffle`    | true    | false    |
| `SubgroupBroadcast`  | true    | false    |
| `SubgroupLocalId`    | true    | true     |
| `Arithmetic`         | true    | true     |
| `Store`              | false   | false    |
