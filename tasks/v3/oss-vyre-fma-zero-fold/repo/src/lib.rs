//! A peephole identity-elimination pass for a two-operand internal IR.
//!
//! Bug: `Fma(a, b, c)` folds to `c` when either `a` or `b` is a literal
//! numeric zero, WITHOUT checking whether the *other* factor is finite.
//! IEEE 754: `0.0 * inf = NaN`, `0.0 * NaN = NaN`, so `Fma(0.0, inf, c) =
//! NaN`, NOT `c`. Folding silently replaces a NaN result with the addend
//! — a miscompile.
//!
//! A correct fix needs one more precondition: either
//!   (a is zero AND b is finite)  or  (b is zero AND a is finite).

/// A value in the operands table that can be either a literal or a result of
/// an operation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Literal(Literal),
    Result(u32),
}

/// The kind of a literal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Literal {
    I32(i32),
    F32(u32), // raw bits — may represent NaN, inf, or normal
}

impl Literal {
    /// Return the 32-bit value interpreted as an f32.
    fn as_f32(self) -> f32 {
        f32::from_bits(match self {
            Literal::I32(v) => v as u32,
            Literal::F32(bits) => bits,
        })
    }

    /// Return true if the literal is a numeric zero (positive or negative).
    pub fn is_numeric_zero(self) -> bool {
        match self {
            Literal::I32(v) => v == 0,
            Literal::F32(bits) => {
                // bit pattern for 0.0 or -0.0
                bits == 0x0000_0000 || bits == 0x8000_0000
            }
        }
    }

    /// Return true if the literal is finite (not NaN, not ±inf).
    /// Integers are always finite.
    pub fn is_finite_numeric(self) -> bool {
        match self {
            Literal::I32(_) => true,
            Literal::F32(bits) => {
                if bits & 0x7F80_0000 == 0x7F80_0000 {
                    // exponent all-ones → NaN or inf → non-finite
                    false
                } else {
                    true
                }
            }
        }
    }
}

/// Operation kinds relevant to identity elimination.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpKind {
    /// Scalar literal (value stored elsewhere)
    Literal,
    /// fused multiply-add: result = a * b + c
    Fma,
    /// Store to memory (to make the test observable)
    Store,
}

/// An operation in the body.
#[derive(Debug, Clone)]
pub struct Op {
    pub kind: OpKind,
    pub operands: Vec<u32>, // operand table indices
    pub result: Option<u32>, // defines a result if Some
}

/// The full body — a sequence of operations.
pub struct Body {
    pub ops: Vec<Op>,
    pub values: Vec<Value>, // indexed via Op.operands/result
}

// --------------------------------------------------------------------
// BUGGY identity-elimination pass
// --------------------------------------------------------------------
//
// The bug: when a factor of Fma is Literal(0), the pass folds
// Fma(a,b,c) → c without checking whether the OTHER factor is
// FINITE. 0.0 * inf = 0.0 * NaN = NaN, so the result of Fma
// with 0.0 × non-finite is NaN, not the addend.
//
// The fix (commented out in the function) adds the finite check.

/// Identify-and-eliminate identity patterns.
///
/// **BUGGY VERSION** – see `identity_elim_fix` for the corrected version.
pub fn identity_elim(body: &mut Body) {
    // Build literal lookup: value-index → the raw Literal if it is a scalar lit.
    let lit: Vec<Option<Literal>> = body
        .values
        .iter()
        .map(|v| match v {
            Value::Literal(lit) => Some(*lit),
            Value::Result(_) => None,
        })
        .collect();

    // Map from a result index to the "effective" value after elimination
    // (i.e. which value index to use instead).
    let mut remap: Vec<Option<u32>> = vec![None; body.values.len()];

    for op in &body.ops {
        if op.kind != OpKind::Fma || op.operands.len() < 3 {
            continue;
        }
        let a_raw = op.operands[0] as usize;
        let b_raw = op.operands[1] as usize;
        let c_raw = op.operands[2] as usize;

        // Resolve through previous remapping.
        let a = resolve(a_raw, &remap);
        let b = resolve(b_raw, &remap);
        let c = resolve(c_raw, &remap);

        let a_lit = lit.get(a as usize).copied().flatten();
        let b_lit = lit.get(b as usize).copied().flatten();

        // ── BUG ──────────────────────────────────────────────
        // Only checks whether a factor is zero; DOES NOT check
        // whether the OTHER factor is finite.  We need both:
        //   (a is zero AND b is finite)  OR  (b is zero AND a is finite)
        let a_zero = a_lit.is_some_and(Literal::is_numeric_zero);
        let b_zero = b_lit.is_some_and(Literal::is_numeric_zero);

        // The fix would add finite checks:
        // let a_finite = a_lit.is_some_and(Literal::is_finite_numeric);
        // let b_finite = b_lit.is_some_and(Literal::is_finite_numeric);
        // if (a_zero && b_finite) || (b_zero && a_finite) {
        //   ... then fold ...
        // }

        if a_zero || b_zero {
            // BUGGY: Fma(a, zero, c) → c  even when `a` is inf/NaN
            if let Some(rid) = op.result {
                remap[rid as usize] = Some(c);
            }
        }
    }

    // Apply remapping to all operand references.
    for op in &mut body.ops {
        for operand in &mut op.operands {
            let resolved = resolve(*operand as usize, &remap);
            *operand = resolved;
        }
    }
}

/// Resolve a value index through remapping transitively.
fn resolve(mut idx: usize, remap: &[Option<u32>]) -> u32 {
    while let Some(next) = remap.get(idx).copied().flatten() {
        idx = next as usize;
    }
    idx as u32
}

// --------------------------------------------------------------------
// Helpers for creating test bodies
// --------------------------------------------------------------------

pub fn literal(val: Literal) -> (Value, u32) {
    (Value::Literal(val), 0) // index filled in by add_values
}

pub trait BodyBuilder {
    fn add(&mut self, op: Op) -> u32;
    fn add_value(&mut self, v: Value) -> u32;
}

impl BodyBuilder for Body {
    fn add(&mut self, op: Op) -> u32 {
        let idx = self.ops.len() as u32;
        self.ops.push(op);
        idx
    }

    fn add_value(&mut self, v: Value) -> u32 {
        let idx = self.values.len() as u32;
        self.values.push(v);
        idx
    }
}

// --------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fma_zero_times_infinity_must_not_fold_to_addend() {
        // Construct:
        //   r0 = Lit(0.0f32)
        //   r1 = Lit(+inf f32)
        //   r2 = Lit(7.0f32)
        //   r3 = Fma(r0, r1, r2)
        //   Store(r3)

        let mut body = Body { ops: vec![], values: vec![] };

        let v0 = body.add_value(Value::Literal(Literal::F32(0x0000_0000))); // 0.0
        let v1 = body.add_value(Value::Literal(Literal::F32(0x7F80_0000))); // +inf
        let v2 = body.add_value(Value::Literal(Literal::F32(0x40E0_0000))); // 7.0

        let fma_result = body.add_value(Value::Result(3)); // will be defined by the Fma

        // Store value index that Fma writes to
        let store_dest = body.add_value(Value::Result(4)); // dummy

        body.add(Op {
            kind: OpKind::Literal,
            operands: vec![0],
            result: Some(0),
        });
        body.add(Op {
            kind: OpKind::Literal,
            operands: vec![1],
            result: Some(1),
        });
        body.add(Op {
            kind: OpKind::Literal,
            operands: vec![2],
            result: Some(2),
        });
        body.add(Op {
            kind: OpKind::Fma,
            operands: vec![0, 1, 2], // a=0.0, b=inf, c=7.0
            result: Some(3),
        });
        body.add(Op {
            kind: OpKind::Store,
            operands: vec![3], // Store(Fma_result)
            result: None,
        });

        identity_elim(&mut body);

        // After buggy identity elimination, the store's operand should
        // still be the Fma result (3), NOT the addend (2).  Only the
        // buggy pass folds Fma(0, inf, 7.0) → 7.0 which, per IEEE 754,
        // is wrong: the result is NaN.
        let store = &body.ops[4];
        assert_eq!(
            store.operands[0], 3,
            "Fma(0.0, inf, 7.0) should NOT fold to addend — result is NaN"
        );
    }
}
