//! lint on blocks unnecessarily using >= with a + 1 or - 1

use rustc::lint::*;
use syntax::ast::*;

use utils::{snippet_opt, span_lint_and_then};

/// **What it does:** Checks for usage of `x >= y + 1` or `x - 1 >= y` (and `<=`) in a block
///
///
/// **Why is this bad?** Readability -- better to use `> y` instead of `>= y + 1`.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// x >= y + 1
/// ```
///
/// Could be written:
///
/// ```rust
/// x > y
/// ```
declare_clippy_lint! {
    pub INT_PLUS_ONE,
    complexity,
    "instead of using x >= y + 1, use x > y"
}

pub struct IntPlusOne;

impl LintPass for IntPlusOne {
    fn get_lints(&self) -> LintArray {
        lint_array!(INT_PLUS_ONE)
    }
}

// cases:
// BinOpKind::Ge
// x >= y + 1
// x - 1 >= y
//
// BinOpKind::Le
// x + 1 <= y
// x <= y - 1

#[derive(Copy, Clone)]
enum Side {
    LHS,
    RHS,
}

impl IntPlusOne {
    #[allow(cast_sign_loss)]
    fn check_lit(&self, lit: &Lit, target_value: i128) -> bool {
        if let LitKind::Int(value, ..) = lit.node {
            return value == (target_value as u128);
        }
        false
    }

    fn check_binop(&self, cx: &EarlyContext, binop: BinOpKind, lhs: &Expr, rhs: &Expr) -> Option<String> {
        match (binop, &lhs.node, &rhs.node) {
            // case where `x - 1 >= ...` or `-1 + x >= ...`
            (BinOpKind::Ge, &ExprKind::Binary(ref lhskind, ref lhslhs, ref lhsrhs), _) => {
                match (lhskind.node, &lhslhs.node, &lhsrhs.node) {
                    // `-1 + x`
                    (BinOpKind::Add, &ExprKind::Lit(ref lit), _) if self.check_lit(lit, -1) => {
                        self.generate_recommendation(cx, binop, lhsrhs, rhs, Side::LHS)
                    },
                    // `x - 1`
                    (BinOpKind::Sub, _, &ExprKind::Lit(ref lit)) if self.check_lit(lit, 1) => {
                        self.generate_recommendation(cx, binop, lhslhs, rhs, Side::LHS)
                    },
                    _ => None,
                }
            },
            // case where `... >= y + 1` or `... >= 1 + y`
            (BinOpKind::Ge, _, &ExprKind::Binary(ref rhskind, ref rhslhs, ref rhsrhs))
                if rhskind.node == BinOpKind::Add =>
            {
                match (&rhslhs.node, &rhsrhs.node) {
                    // `y + 1` and `1 + y`
                    (&ExprKind::Lit(ref lit), _) if self.check_lit(lit, 1) => {
                        self.generate_recommendation(cx, binop, rhsrhs, lhs, Side::RHS)
                    },
                    (_, &ExprKind::Lit(ref lit)) if self.check_lit(lit, 1) => {
                        self.generate_recommendation(cx, binop, rhslhs, lhs, Side::RHS)
                    },
                    _ => None,
                }
            }
            // case where `x + 1 <= ...` or `1 + x <= ...`
            (BinOpKind::Le, &ExprKind::Binary(ref lhskind, ref lhslhs, ref lhsrhs), _)
                if lhskind.node == BinOpKind::Add =>
            {
                match (&lhslhs.node, &lhsrhs.node) {
                    // `1 + x` and `x + 1`
                    (&ExprKind::Lit(ref lit), _) if self.check_lit(lit, 1) => {
                        self.generate_recommendation(cx, binop, lhsrhs, rhs, Side::LHS)
                    },
                    (_, &ExprKind::Lit(ref lit)) if self.check_lit(lit, 1) => {
                        self.generate_recommendation(cx, binop, lhslhs, rhs, Side::LHS)
                    },
                    _ => None,
                }
            }
            // case where `... >= y - 1` or `... >= -1 + y`
            (BinOpKind::Le, _, &ExprKind::Binary(ref rhskind, ref rhslhs, ref rhsrhs)) => {
                match (rhskind.node, &rhslhs.node, &rhsrhs.node) {
                    // `-1 + y`
                    (BinOpKind::Add, &ExprKind::Lit(ref lit), _) if self.check_lit(lit, -1) => {
                        self.generate_recommendation(cx, binop, rhsrhs, lhs, Side::RHS)
                    },
                    // `y - 1`
                    (BinOpKind::Sub, _, &ExprKind::Lit(ref lit)) if self.check_lit(lit, 1) => {
                        self.generate_recommendation(cx, binop, rhslhs, lhs, Side::RHS)
                    },
                    _ => None,
                }
            },
            _ => None,
        }
    }

    fn generate_recommendation(
        &self,
        cx: &EarlyContext,
        binop: BinOpKind,
        node: &Expr,
        other_side: &Expr,
        side: Side,
    ) -> Option<String> {
        let binop_string = match binop {
            BinOpKind::Ge => ">",
            BinOpKind::Le => "<",
            _ => return None,
        };
        if let Some(snippet) = snippet_opt(cx, node.span) {
            if let Some(other_side_snippet) = snippet_opt(cx, other_side.span) {
                let rec = match side {
                    Side::LHS => Some(format!("{} {} {}", snippet, binop_string, other_side_snippet)),
                    Side::RHS => Some(format!("{} {} {}", other_side_snippet, binop_string, snippet)),
                };
                return rec;
            }
        }
        None
    }

    fn emit_warning(&self, cx: &EarlyContext, block: &Expr, recommendation: String) {
        span_lint_and_then(cx, INT_PLUS_ONE, block.span, "Unnecessary `>= y + 1` or `x - 1 >=`", |db| {
            db.span_suggestion(block.span, "change `>= y + 1` to `> y` as shown", recommendation);
        });
    }
}

impl EarlyLintPass for IntPlusOne {
    fn check_expr(&mut self, cx: &EarlyContext, item: &Expr) {
        if let ExprKind::Binary(ref kind, ref lhs, ref rhs) = item.node {
            if let Some(ref rec) = self.check_binop(cx, kind.node, lhs, rhs) {
                self.emit_warning(cx, item, rec.clone());
            }
        }
    }
}
