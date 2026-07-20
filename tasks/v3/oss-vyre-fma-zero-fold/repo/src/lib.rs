//! Peephole identity-elimination for a small internal IR.

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
                // exponent all-ones → NaN or inf → non-finite
                bits & 0x7F80_0000 != 0x7F80_0000
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

/// Identify-and-eliminate identity patterns.
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

        let a_zero = a_lit.is_some_and(Literal::is_numeric_zero);
        let b_zero = b_lit.is_some_and(Literal::is_numeric_zero);

        if a_zero || b_zero {
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
