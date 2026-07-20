// pass_to_pass: identity_elim correctly folds Fma where both factors are finite.
// These must keep working before AND after the fix.

use oss_vyre_fma_zero_fold::*;

#[test]
fn fma_zero_times_finite_folds_to_addend() {
    // Fma(0.0, 42.0, 7.0)  →  0.0*42.0 + 7.0  =  7.0  → should fold to addend
    let mut body = Body { ops: vec![], values: vec![] };
    body.add_value(Value::Literal(Literal::F32(0x0000_0000))); // v0: 0.0
    body.add_value(Value::Literal(Literal::F32(0x4228_0000))); // v1: 42.0 (finite)
    body.add_value(Value::Literal(Literal::F32(0x40E0_0000))); // v2: 7.0
    body.add_value(Value::Result(3));
    body.add_value(Value::Result(4));
    body.add(Op { kind: OpKind::Literal, operands: vec![0], result: Some(0) });
    body.add(Op { kind: OpKind::Literal, operands: vec![1], result: Some(1) });
    body.add(Op { kind: OpKind::Literal, operands: vec![2], result: Some(2) });
    body.add(Op { kind: OpKind::Fma, operands: vec![0, 1, 2], result: Some(3) });
    body.add(Op { kind: OpKind::Store, operands: vec![3], result: None });

    identity_elim(&mut body);
    let store = &body.ops[4];
    // With both factors zero/finite, the fold IS valid: operand becomes addend (2).
    assert_eq!(
        store.operands[0], 2,
        "Fma(0.0, 42.0, 7.0) should fold to addend"
    );
}

#[test]
fn fma_finite_times_zero_folds_to_addend() {
    // Fma(42.0, 0.0, 7.0) — switched operand order
    let mut body = Body { ops: vec![], values: vec![] };
    body.add_value(Value::Literal(Literal::F32(0x4228_0000))); // v0: 42.0
    body.add_value(Value::Literal(Literal::F32(0x0000_0000))); // v1: 0.0
    body.add_value(Value::Literal(Literal::F32(0x40E0_0000))); // v2: 7.0
    body.add_value(Value::Result(3));
    body.add_value(Value::Result(4));
    body.add(Op { kind: OpKind::Literal, operands: vec![0], result: Some(0) });
    body.add(Op { kind: OpKind::Literal, operands: vec![1], result: Some(1) });
    body.add(Op { kind: OpKind::Literal, operands: vec![2], result: Some(2) });
    body.add(Op { kind: OpKind::Fma, operands: vec![0, 1, 2], result: Some(3) });
    body.add(Op { kind: OpKind::Store, operands: vec![3], result: None });

    identity_elim(&mut body);
    let store = &body.ops[4];
    assert_eq!(
        store.operands[0], 2,
        "Fma(42.0, 0.0, 7.0) should fold to addend"
    );
}

#[test]
fn fma_with_non_literal_factor_not_folded() {
    // When neither factor is a literal, identity_elim should NOT fold.
    let mut body = Body { ops: vec![], values: vec![] };
    body.add_value(Value::Literal(Literal::F32(0x40E0_0000))); // v0: 7.0 (addend)
    body.add_value(Value::Result(1)); // v1: some value (non-literal a)
    body.add_value(Value::Result(2)); // v2: some value (non-literal b)
    body.add_value(Value::Result(3)); // v3: Fma result
    body.add_value(Value::Result(4));
    body.add(Op { kind: OpKind::Literal, operands: vec![0], result: Some(0) });
    // a and b are non-literal — identity_elim can't prove zero or finite
    body.add(Op { kind: OpKind::Fma, operands: vec![1, 2, 0], result: Some(3) });
    body.add(Op { kind: OpKind::Store, operands: vec![3], result: None });

    identity_elim(&mut body);
    let store = &body.ops[2];
    assert_eq!(
        store.operands[0], 3,
        "Fma with non-literal factors should NOT be folded"
    );
}

#[test]
fn fma_integer_zero_always_folds() {
    // Integers are always finite, so Fma(0i32, x, c) → c is always valid.
    let mut body = Body { ops: vec![], values: vec![] };
    body.add_value(Value::Literal(Literal::I32(0)));     // v0: 0 (integer)
    body.add_value(Value::Literal(Literal::I32(42)));    // v1: 42 (integer)
    body.add_value(Value::Literal(Literal::I32(7)));     // v2: 7
    body.add_value(Value::Result(3));
    body.add_value(Value::Result(4));
    body.add(Op { kind: OpKind::Literal, operands: vec![0], result: Some(0) });
    body.add(Op { kind: OpKind::Literal, operands: vec![1], result: Some(1) });
    body.add(Op { kind: OpKind::Literal, operands: vec![2], result: Some(2) });
    body.add(Op { kind: OpKind::Fma, operands: vec![0, 1, 2], result: Some(3) });
    body.add(Op { kind: OpKind::Store, operands: vec![3], result: None });

    identity_elim(&mut body);
    let store = &body.ops[4];
    assert_eq!(
        store.operands[0], 2,
        "Fma(0i32, 42i32, 7i32) should fold to addend (integers always finite)"
    );
}
