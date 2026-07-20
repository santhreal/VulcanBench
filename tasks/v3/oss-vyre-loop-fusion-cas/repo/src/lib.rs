//! Scalar-dependency analysis for loop fusion.
//!
//! Bug: `collect_vars_in_expr` collects the variables referenced by an
//! expression, used by the scalar-dependency gate `fusion_has_scalar_dependency`
//! that blocks loop fusion when two loops read/write the same scalar. The
//! `Expr::Atomic` arm collects only `index` and `value` but **not** `expected` —
//! the compare-exchange's expected operand. Since `expected` is a SCALAR READ
//! (the CAS reads the variable to compare against), omitting it lets the
//! dependency analysis miss a cross-loop scalar read, silently allowing two loops
//! to be fused when the first mutates a scalar that the second CAS's `expected`
//! reads.
//!
//! The fix has two parts:
//!   1. Add `expected` to the collected vars in `Expr::Atomic`.
//!   2. Replace the `_ => {}` catch-all with exhaustive leaf variants so new
//!      operand-bearing `Expr` variants are explicitly wired in, not silently
//!      dropped.
//!
//! (The real bug was vyre-foundation's loop fusion pass, commit 0685fea4.)

use std::collections::HashSet;

/// A variable name.
pub type Ident = &'static str;

/// Minimal expressions matching the relevant structure from vyre-foundation's IR.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literal value.
    LitU32(u32),
    /// Variable reference.
    Var(Ident),
    /// Add two sub-expressions.
    Add(Box<Expr>, Box<Expr>),
    /// Atomic compare-exchange: `atomicCAS(addr, expected, value)`.
    Atomic {
        index: Box<Expr>,
        expected: Box<Expr>,
        value: Box<Expr>,
    },
}

/// A statement in a loop body.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Assign expression result to a variable.
    Assign(Ident, Expr),
}

/// A loop body.
#[derive(Debug, Clone)]
pub struct LoopBody {
    pub stmts: Vec<Stmt>,
}

// --------------------------------------------------------------------
// BUGGY: `collect_vars_in_expr` — missing `expected` in Atomic
// --------------------------------------------------------------------

/// Collect all variables referenced by an expression.
///
/// **BUGGY VERSION**: the `Expr::Atomic` arm collects `index` and `value`
/// but NOT `expected`. Since `expected` is a scalar read of a variable,
/// this omission lets the dependency analysis miss cross-loop scalar reads.
pub fn collect_vars_in_expr(expr: &Expr, out: &mut HashSet<Ident>) {
    match expr {
        Expr::LitU32(_) => {}
        Expr::Var(name) => {
            out.insert(name);
        }
        Expr::Add(a, b) => {
            collect_vars_in_expr(a, out);
            collect_vars_in_expr(b, out);
        }
        Expr::Atomic {
            index,
            expected: _, // BUG: expected is NOT collected!
            value,
        } => {
            collect_vars_in_expr(index, out);
            // BUG: missing: collect_vars_in_expr(expected, out);
            collect_vars_in_expr(value, out);
        }
        // Note: the catch-all `_ => {}` variant is also a latent bug pattern
        // (new Expr variants silently get zero analysis), but this is a
        // secondary concern.
    }
}
/// Return true if fusing two loops would reorder a scalar read/write.
///
/// `loop_a` runs before `loop_b`. If `loop_b` reads a scalar that `loop_a`
/// writes, fusion is blocked (the interleaving would see different values).
pub fn fusion_has_scalar_dependency(a: &LoopBody, b: &LoopBody) -> bool {
    // Collect all writes in loop_a.
    let mut writes_a: HashSet<Ident> = HashSet::new();
    for stmt in &a.stmts {
        let Stmt::Assign(name, _) = stmt;
        writes_a.insert(name);
    }

    // Collect all reads in loop_b.
    let mut reads_b: HashSet<Ident> = HashSet::new();
    for stmt in &b.stmts {
        let Stmt::Assign(_, expr) = stmt;
        collect_vars_in_expr(expr, &mut reads_b);
    }

    // If any value written by A is read by B, there's a dependency.
    writes_a.intersection(&reads_b).next().is_some()
}
