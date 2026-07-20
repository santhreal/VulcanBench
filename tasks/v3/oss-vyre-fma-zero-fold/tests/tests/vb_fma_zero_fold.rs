// fail_to_pass: identity_elim must not fold Fma(0.0, inf, c) → c.
// IEEE 754: 0.0 * inf = NaN, so Fma(0.0, inf, 7.0) = NaN, NOT 7.0.
// Unfixed: the pass collapses Fma(0, any, c) → c regardless of the
// other factor.
//
// The test constructs:
//   r0 = Lit(0.0f32),  r1 = Lit(+inf f32),  r2 = Lit(7.0f32),
//   r3 = Fma(r0, r1, r2),  Store(r3)
//
// After buggy identity_elim, the Store operand is 2 (folded to addend)
// instead of 3 (should remain the Fma result).
use oss_vyre_fma_zero_fold::*;

fn make_body_zero_times_inf() -> Body {
    let mut body = Body { ops: vec![], values: vec![] };
    body.add_value(Value::Literal(Literal::F32(0x0000_0000))); // v0: 0.0
    body.add_value(Value::Literal(Literal::F32(0x7F80_0000))); // v1: +inf
    body.add_value(Value::Literal(Literal::F32(0x40E0_0000))); // v2: 7.0
    body.add_value(Value::Result(3));                          // v3: Fma result
    body.add_value(Value::Result(4));                          // v4: store dest (dummy)
    body.add(Op { kind: OpKind::Literal, operands: vec![0], result: Some(0) });
    body.add(Op { kind: OpKind::Literal, operands: vec![1], result: Some(1) });
    body.add(Op { kind: OpKind::Literal, operands: vec![2], result: Some(2) });
    body.add(Op { kind: OpKind::Fma, operands: vec![0, 1, 2], result: Some(3) });
    body.add(Op { kind: OpKind::Store, operands: vec![3], result: None });
    body
}

#[test]
fn fma_zero_times_inf_must_not_fold() {
    let mut body = make_body_zero_times_inf();
    identity_elim(&mut body);
    // Store operand should still be Fma result (3), not addend (2).
    let store = &body.ops[4];
    assert_eq!(
        store.operands[0], 3,
        "Fma(0.0, inf, 7.0) must not fold to addend: result is NaN, not 7.0"
    );
}

#[test]
fn fma_zero_times_nan_must_not_fold() {
    let mut body = Body { ops: vec![], values: vec![] };
    body.add_value(Value::Literal(Literal::F32(0x0000_0000))); // v0: 0.0
    body.add_value(Value::Literal(Literal::F32(0x7FC0_0000))); // v1: NaN
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
        store.operands[0], 3,
        "Fma(0.0, NaN, 7.0) must not fold to addend: result is NaN, not 7.0"
    );
}

#[test]
fn fma_neg_zero_times_inf_must_not_fold() {
    // -0.0 * inf = NaN as well
    let mut body = Body { ops: vec![], values: vec![] };
    body.add_value(Value::Literal(Literal::F32(0x8000_0000))); // v0: -0.0
    body.add_value(Value::Literal(Literal::F32(0x7F80_0000))); // v1: +inf
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
        store.operands[0], 3,
        "Fma(-0.0, inf, 7.0) must not fold to addend: result is NaN, not 7.0"
    );
}
