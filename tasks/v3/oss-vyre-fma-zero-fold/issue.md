# `identity_elim` incorrectly folds `Fma(0.0, inf, c) → c`

The peephole identity-elimination pass collapses `Fma(a, b, c) → c` whenever one
factor is a literal numeric zero. But IEEE 754 arithmetic says `0.0 * inf = NaN`
and `0.0 * NaN = NaN`, so `Fma(0.0, inf, 7.0)` evaluates to `NaN`, not `7.0`.
Folding it to the addend is a miscompile — the result `NaN` is silently replaced
with `c`.

The pass only checks whether a factor is a literal zero (`a_zero || b_zero`) but
does **not** check whether the *other* factor is finite.  A correct fold requires
both conditions:
- one factor is a literal numeric zero **AND**
- the other factor is a finite literal (`is_finite_numeric`).

## Expected behavior

`identity_elim` must only fold `Fma` to the addend when one factor is a literal
zero AND the other is a finite literal.  Cases where the other factor is
`inf`/`NaN` must NOT be folded.

## Acceptance examples

- `Fma(0.0, inf, 7.0)` → `identity_elim` must NOT fold; the Fma result must be preserved
- `Fma(0.0, NaN, 7.0)` → must NOT fold (same reason)
- `Fma(0.0, 42.0, 7.0)` → MUST fold to addend (both factors zero/finite)
- `Fma(42.0, 0.0, 7.0)` → MUST fold to addend (switched order)
