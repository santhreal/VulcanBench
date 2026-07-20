//! Scalar-dependency analysis for loop fusion.

use std::collections::HashSet;

/// A variable name.
pub type Ident = &'static str;

/// Minimal expressions matching the relevant structure from an internal IR.
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

/// Collect all variables referenced by an expression.
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
            expected: _,
            value,
        } => {
            collect_vars_in_expr(index, out);
            collect_vars_in_expr(value, out);
        }
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
