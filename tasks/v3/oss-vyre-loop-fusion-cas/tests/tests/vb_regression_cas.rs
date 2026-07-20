// pass_to_pass: regression guards that hold before AND after the fix.

use oss_vyre_loop_fusion_cas::*;

#[test]
fn non_atomic_scalar_dep_still_detected() {
    // Loop A writes "a", Loop B reads "a" via a plain Add expression.
    let loop_a = LoopBody {
        stmts: vec![Stmt::Assign("a", Expr::LitU32(1))],
    };

    let loop_b = LoopBody {
        stmts: vec![Stmt::Assign(
            "b",
            Expr::Add(Box::new(Expr::Var("a")), Box::new(Expr::LitU32(1))),
        )],
    };

    assert!(
        fusion_has_scalar_dependency(&loop_a, &loop_b),
        "plain Var read must still be detected"
    );
    assert!(
        !fusion_has_scalar_dependency(&loop_b, &loop_a),
        "reverse direction must have no dependency"
    );
}

#[test]
fn no_dep_on_disjoint_scalars() {
    // Loop A writes "x", Loop B writes "y" — no overlap.
    let loop_a = LoopBody {
        stmts: vec![Stmt::Assign("x", Expr::LitU32(1))],
    };

    let loop_b = LoopBody {
        stmts: vec![Stmt::Assign("y", Expr::LitU32(2))],
    };

    assert!(
        !fusion_has_scalar_dependency(&loop_a, &loop_b),
        "disjoint scalars must have no dependency"
    );
}

#[test]
fn cas_without_expected_var_must_not_false_positive() {
    // If loops use disjoint variables, even with a CAS there's no dep.
    let loop_a = LoopBody {
        stmts: vec![Stmt::Assign("x", Expr::LitU32(1))],
    };

    let loop_b = LoopBody {
        stmts: vec![Stmt::Assign(
            "y",
            Expr::Atomic {
                index: Box::new(Expr::Var("ptr")),
                expected: Box::new(Expr::LitU32(0)),
                value: Box::new(Expr::Var("v")),
            },
        )],
    };

    assert!(
        !fusion_has_scalar_dependency(&loop_a, &loop_b),
        "CAS with literal expected and disjoint vars — no dependency"
    );
}
