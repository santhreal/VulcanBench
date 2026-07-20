# Loop fusion misses CAS `expected` operand as scalar dependency

`fusion_has_scalar_dependency` blocks loop fusion when loop A writes a scalar
that loop B reads (because the interleaving would reorder the access). The
`collect_vars_in_expr` function collects which variables an expression reads,
used by this dependency analysis.

When the expression is an atomic compare-exchange
(`Expr::Atomic { index, expected, value }`), the buggy code collects only `index`
and `value` — it does **not** collect `expected`. The `expected` operand is a
scalar read (the CAS compares the memory value against it), so omitting it lets
the dependency analysis miss a cross-loop scalar read.

Example:
- Loop A: `a = x + 1` (writes variable `a`)
- Loop B: `y = atomicCAS(ptr, expected=a, value=b)` (reads variable `a` via `expected`)

The buggy analysis reports no dependency, allowing the two loops to be fused.
When fused, the interleaved execution may see an incorrect value for `a` inside
the CAS.

The fix:
1. collect `expected` in `Expr::Atomic`
2. Remove the catch-all `_ => {}` arm (use exhaustive variant matching)

## Expected behavior

`fusion_has_scalar_dependency` must return `true` when one loop body writes a
variable that another loop body's CAS `expected` operand reads.
