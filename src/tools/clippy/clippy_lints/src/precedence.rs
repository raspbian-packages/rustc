use rustc::lint::*;
use syntax::ast::*;
use syntax::codemap::Spanned;
use utils::{span_lint_and_sugg, snippet};

/// **What it does:** Checks for operations where precedence may be unclear
/// and suggests to add parentheses. Currently it catches the following:
/// * mixed usage of arithmetic and bit shifting/combining operators without
/// parentheses
/// * a "negative" numeric literal (which is really a unary `-` followed by a
/// numeric literal)
///   followed by a method call
///
/// **Why is this bad?** Not everyone knows the precedence of those operators by
/// heart, so expressions like these may trip others trying to reason about the
/// code.
///
/// **Known problems:** None.
///
/// **Example:**
/// * `1 << 2 + 3` equals 32, while `(1 << 2) + 3` equals 7
/// * `-1i32.abs()` equals -1, while `(-1i32).abs()` equals 1
declare_lint! {
    pub PRECEDENCE,
    Warn,
    "operations where precedence may be unclear"
}

#[derive(Copy, Clone)]
pub struct Precedence;

impl LintPass for Precedence {
    fn get_lints(&self) -> LintArray {
        lint_array!(PRECEDENCE)
    }
}

impl EarlyLintPass for Precedence {
    fn check_expr(&mut self, cx: &EarlyContext, expr: &Expr) {
        if let ExprKind::Binary(Spanned { node: op, .. }, ref left, ref right) = expr.node {
            let span_sugg = |expr: &Expr, sugg| {
                span_lint_and_sugg(
                    cx,
                    PRECEDENCE,
                    expr.span,
                    "operator precedence can trip the unwary",
                    "consider parenthesizing your expression",
                    sugg,
                );
            };

            if !is_bit_op(op) {
                return;
            }
            match (is_arith_expr(left), is_arith_expr(right)) {
                (true, true) => {
                    let sugg = format!(
                        "({}) {} ({})",
                        snippet(cx, left.span, ".."),
                        op.to_string(),
                        snippet(cx, right.span, "..")
                    );
                    span_sugg(expr, sugg);
                },
                (true, false) => {
                    let sugg = format!(
                        "({}) {} {}",
                        snippet(cx, left.span, ".."),
                        op.to_string(),
                        snippet(cx, right.span, "..")
                    );
                    span_sugg(expr, sugg);
                },
                (false, true) => {
                    let sugg = format!(
                        "{} {} ({})",
                        snippet(cx, left.span, ".."),
                        op.to_string(),
                        snippet(cx, right.span, "..")
                    );
                    span_sugg(expr, sugg);
                },
                (false, false) => (),
            }
        }

        if let ExprKind::Unary(UnOp::Neg, ref rhs) = expr.node {
            if let ExprKind::MethodCall(_, ref args) = rhs.node {
                if let Some(slf) = args.first() {
                    if let ExprKind::Lit(ref lit) = slf.node {
                        match lit.node {
                            LitKind::Int(..) |
                            LitKind::Float(..) |
                            LitKind::FloatUnsuffixed(..) => {
                                span_lint_and_sugg(
                                    cx,
                                    PRECEDENCE,
                                    expr.span,
                                    "unary minus has lower precedence than method call",
                                    "consider adding parentheses to clarify your intent",
                                    format!("-({})", snippet(cx, rhs.span, "..")),
                                );
                            },
                            _ => (),
                        }
                    }
                }
            }
        }
    }
}

fn is_arith_expr(expr: &Expr) -> bool {
    match expr.node {
        ExprKind::Binary(Spanned { node: op, .. }, _, _) => is_arith_op(op),
        _ => false,
    }
}

fn is_bit_op(op: BinOpKind) -> bool {
    use syntax::ast::BinOpKind::*;
    match op {
        BitXor | BitAnd | BitOr | Shl | Shr => true,
        _ => false,
    }
}

fn is_arith_op(op: BinOpKind) -> bool {
    use syntax::ast::BinOpKind::*;
    match op {
        Add | Sub | Mul | Div | Rem => true,
        _ => false,
    }
}
