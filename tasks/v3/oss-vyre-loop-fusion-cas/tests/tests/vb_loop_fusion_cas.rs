// fail_to_pass: `fusion_has_scalar_dependency` must detect cross-loop
// scalar reads via the CAS `expected` operand.
//
// Loop_A writes variable "a", Loop_B reads "a" via a CAS `expected` operand.
// Buggy `collect_vars_in_expr` misses `expected`, so the dependency is not
// detected and the loops would be incorrectly fused.

use oss_vyre_loop_fusion_cas::*;

#[test]
fn cas_expected_creates_scalar_dependency() {
    // Loop A: a = x + 1  (writes "a")
    let loop_a = LoopBody {
        stmts: vec![Stmt::Assign("a", Expr::Add(
            Box::new(Expr::Var("x")),
            Box::new(Expr::LitU32(1)),
        ))],
    };

    // Loop B: y = atomicCAS(ptr, expected=a, value=b)  (reads "a")
    let loop_b = LoopBody {
        stmts: vec![Stmt::Assign(
            "y",
            Expr::Atomic {
                index: Box::new(Expr::Var("ptr")),
                expected: Box::new(Expr::Var("a")),
                value: Box::new(Expr::Var("b")),
            },
        )],
    };

    assert!(
        fusion_has_scalar_dependency(&loop_a, &loop_b),
        "loop_b reads 'a' via CAS expected — dependency must be detected"
    );
}

#[test]
fn cas_expected_creates_dep_swapped_loops() {
    // Same dependency but loops in swapped order: Loop_B before Loop_A
    let loop_a = LoopBody {
        stmts: vec![Stmt::Assign("a", Expr::Add(
            Box::new(Expr::Var("x")),
            Box::new(Expr::LitU32(1)),
        ))],
    };

    let loop_b = LoopBody {
        stmts: vec![Stmt::Assign(
            "y",
            Expr::Atomic {
                index: Box::new(Expr::Var("ptr")),
                expected: Box::new(Expr::Var("a")),
                value: Box::new(Expr::Var("b")),
            },
        )],
    };

    // loop_b (CAS) runs first, loop_a (write) runs second:
    // loop_b reads 'a', loop_a writes 'a' — no dependency in this direction
    // (the first loop can't read something the second writes)
    assert!(
        !fusion_has_scalar_dependency(&loop_b, &loop_a),
        "loop_a writes after loop_b reads — no backward dependency"
    );
}

#[test]
fn cas_expected_is_separate_from_value_operand() {
    // The `expected` operand is distinct from the `value` operand.
    // A reads "val", B reads "exp" via CAS expected — should detect dep.
    let loop_a = LoopBody {
        stmts: vec![Stmt::Assign("exp", Expr::LitU32(42))],
    };

    let loop_b = LoopBody {
        stmts: vec![Stmt::Assign(
            "y",
            Expr::Atomic {
                index: Box::new(Expr::Var("ptr")),
                expected: Box::new(Expr::Var("exp")),
                value: Box::new(Expr::Var("other")),
            },
        )],
    };

    assert!(
        fusion_has_scalar_dependency(&loop_a, &loop_b),
        "CAS expected reads 'exp' written by loop_a — dependency must be detected"
    );
}
