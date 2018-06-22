use rustc::hir::*;
use rustc::lint::*;
use syntax::ast::LitKind;
use syntax::codemap::Span;
use utils::{span_lint, span_lint_and_then};
use utils::sugg::Sugg;
use consts::{constant, Constant};

/// **What it does:** Checks for incompatible bit masks in comparisons.
///
/// The formula for detecting if an expression of the type `_ <bit_op> m
/// <cmp_op> c` (where `<bit_op>` is one of {`&`, `|`} and `<cmp_op>` is one of
/// {`!=`, `>=`, `>`, `!=`, `>=`, `>`}) can be determined from the following
/// table:
///
/// |Comparison  |Bit Op|Example     |is always|Formula               |
/// |------------|------|------------|---------|----------------------|
/// |`==` or `!=`| `&`  |`x & 2 == 3`|`false`  |`c & m != c`          |
/// |`<`  or `>=`| `&`  |`x & 2 < 3` |`true`   |`m < c`               |
/// |`>`  or `<=`| `&`  |`x & 1 > 1` |`false`  |`m <= c`              |
/// |`==` or `!=`| `|`  |`x | 1 == 0`|`false`  |`c | m != c`          |
/// |`<`  or `>=`| `|`  |`x | 1 < 1` |`false`  |`m >= c`              |
/// |`<=` or `>` | `|`  |`x | 1 > 0` |`true`   |`m > c`               |
///
/// **Why is this bad?** If the bits that the comparison cares about are always
/// set to zero or one by the bit mask, the comparison is constant `true` or
/// `false` (depending on mask, compared value, and operators).
///
/// So the code is actively misleading, and the only reason someone would write
/// this intentionally is to win an underhanded Rust contest or create a
/// test-case for this lint.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// if (x & 1 == 2) { … }
/// ```
declare_clippy_lint! {
    pub BAD_BIT_MASK,
    correctness,
    "expressions of the form `_ & mask == select` that will only ever return `true` or `false`"
}

/// **What it does:** Checks for bit masks in comparisons which can be removed
/// without changing the outcome. The basic structure can be seen in the
/// following table:
///
/// |Comparison| Bit Op  |Example    |equals |
/// |----------|---------|-----------|-------|
/// |`>` / `<=`|`|` / `^`|`x | 2 > 3`|`x > 3`|
/// |`<` / `>=`|`|` / `^`|`x ^ 1 < 4`|`x < 4`|
///
/// **Why is this bad?** Not equally evil as [`bad_bit_mask`](#bad_bit_mask),
/// but still a bit misleading, because the bit mask is ineffective.
///
/// **Known problems:** False negatives: This lint will only match instances
/// where we have figured out the math (which is for a power-of-two compared
/// value). This means things like `x | 1 >= 7` (which would be better written
/// as `x >= 6`) will not be reported (but bit masks like this are fairly
/// uncommon).
///
/// **Example:**
/// ```rust
/// if (x | 1 > 3) { … }
/// ```
declare_clippy_lint! {
    pub INEFFECTIVE_BIT_MASK,
    correctness,
    "expressions where a bit mask will be rendered useless by a comparison, e.g. `(x | 1) > 2`"
}

/// **What it does:** Checks for bit masks that can be replaced by a call
/// to `trailing_zeros`
///
/// **Why is this bad?** `x.trailing_zeros() > 4` is much clearer than `x & 15
/// == 0`
///
/// **Known problems:** llvm generates better code for `x & 15 == 0` on x86
///
/// **Example:**
/// ```rust
/// x & 0x1111 == 0
/// ```
declare_clippy_lint! {
    pub VERBOSE_BIT_MASK,
    style,
    "expressions where a bit mask is less readable than the corresponding method call"
}

#[derive(Copy, Clone)]
pub struct BitMask {
    verbose_bit_mask_threshold: u64,
}

impl BitMask {
    pub fn new(verbose_bit_mask_threshold: u64) -> Self {
        Self {
            verbose_bit_mask_threshold,
        }
    }
}

impl LintPass for BitMask {
    fn get_lints(&self) -> LintArray {
        lint_array!(BAD_BIT_MASK, INEFFECTIVE_BIT_MASK, VERBOSE_BIT_MASK)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for BitMask {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, e: &'tcx Expr) {
        if let ExprBinary(ref cmp, ref left, ref right) = e.node {
            if cmp.node.is_comparison() {
                if let Some(cmp_opt) = fetch_int_literal(cx, right) {
                    check_compare(cx, left, cmp.node, cmp_opt, &e.span)
                } else if let Some(cmp_val) = fetch_int_literal(cx, left) {
                    check_compare(cx, right, invert_cmp(cmp.node), cmp_val, &e.span)
                }
            }
        }
        if_chain! {
            if let Expr_::ExprBinary(ref op, ref left, ref right) = e.node;
            if BinOp_::BiEq == op.node;
            if let Expr_::ExprBinary(ref op1, ref left1, ref right1) = left.node;
            if BinOp_::BiBitAnd == op1.node;
            if let Expr_::ExprLit(ref lit) = right1.node;
            if let LitKind::Int(n, _) = lit.node;
            if let Expr_::ExprLit(ref lit1) = right.node;
            if let LitKind::Int(0, _) = lit1.node;
            if n.leading_zeros() == n.count_zeros();
            if n > u128::from(self.verbose_bit_mask_threshold);
            then {
                span_lint_and_then(cx,
                                   VERBOSE_BIT_MASK,
                                   e.span,
                                   "bit mask could be simplified with a call to `trailing_zeros`",
                                   |db| {
                    let sugg = Sugg::hir(cx, left1, "...").maybe_par();
                    db.span_suggestion(e.span, "try", format!("{}.trailing_zeros() >= {}", sugg, n.count_ones()));
                });
            }
        }
    }
}

fn invert_cmp(cmp: BinOp_) -> BinOp_ {
    match cmp {
        BiEq => BiEq,
        BiNe => BiNe,
        BiLt => BiGt,
        BiGt => BiLt,
        BiLe => BiGe,
        BiGe => BiLe,
        _ => BiOr, // Dummy
    }
}


fn check_compare(cx: &LateContext, bit_op: &Expr, cmp_op: BinOp_, cmp_value: u128, span: &Span) {
    if let ExprBinary(ref op, ref left, ref right) = bit_op.node {
        if op.node != BiBitAnd && op.node != BiBitOr {
            return;
        }
        fetch_int_literal(cx, right)
            .or_else(|| fetch_int_literal(cx, left))
            .map_or((), |mask| check_bit_mask(cx, op.node, cmp_op, mask, cmp_value, span))
    }
}

fn check_bit_mask(cx: &LateContext, bit_op: BinOp_, cmp_op: BinOp_, mask_value: u128, cmp_value: u128, span: &Span) {
    match cmp_op {
        BiEq | BiNe => match bit_op {
            BiBitAnd => if mask_value & cmp_value != cmp_value {
                if cmp_value != 0 {
                    span_lint(
                        cx,
                        BAD_BIT_MASK,
                        *span,
                        &format!(
                            "incompatible bit mask: `_ & {}` can never be equal to `{}`",
                            mask_value,
                            cmp_value
                        ),
                    );
                }
            } else if mask_value == 0 {
                span_lint(cx, BAD_BIT_MASK, *span, "&-masking with zero");
            },
            BiBitOr => if mask_value | cmp_value != cmp_value {
                span_lint(
                    cx,
                    BAD_BIT_MASK,
                    *span,
                    &format!(
                        "incompatible bit mask: `_ | {}` can never be equal to `{}`",
                        mask_value,
                        cmp_value
                    ),
                );
            },
            _ => (),
        },
        BiLt | BiGe => match bit_op {
            BiBitAnd => if mask_value < cmp_value {
                span_lint(
                    cx,
                    BAD_BIT_MASK,
                    *span,
                    &format!(
                        "incompatible bit mask: `_ & {}` will always be lower than `{}`",
                        mask_value,
                        cmp_value
                    ),
                );
            } else if mask_value == 0 {
                span_lint(cx, BAD_BIT_MASK, *span, "&-masking with zero");
            },
            BiBitOr => if mask_value >= cmp_value {
                span_lint(
                    cx,
                    BAD_BIT_MASK,
                    *span,
                    &format!(
                        "incompatible bit mask: `_ | {}` will never be lower than `{}`",
                        mask_value,
                        cmp_value
                    ),
                );
            } else {
                check_ineffective_lt(cx, *span, mask_value, cmp_value, "|");
            },
            BiBitXor => check_ineffective_lt(cx, *span, mask_value, cmp_value, "^"),
            _ => (),
        },
        BiLe | BiGt => match bit_op {
            BiBitAnd => if mask_value <= cmp_value {
                span_lint(
                    cx,
                    BAD_BIT_MASK,
                    *span,
                    &format!(
                        "incompatible bit mask: `_ & {}` will never be higher than `{}`",
                        mask_value,
                        cmp_value
                    ),
                );
            } else if mask_value == 0 {
                span_lint(cx, BAD_BIT_MASK, *span, "&-masking with zero");
            },
            BiBitOr => if mask_value > cmp_value {
                span_lint(
                    cx,
                    BAD_BIT_MASK,
                    *span,
                    &format!(
                        "incompatible bit mask: `_ | {}` will always be higher than `{}`",
                        mask_value,
                        cmp_value
                    ),
                );
            } else {
                check_ineffective_gt(cx, *span, mask_value, cmp_value, "|");
            },
            BiBitXor => check_ineffective_gt(cx, *span, mask_value, cmp_value, "^"),
            _ => (),
        },
        _ => (),
    }
}

fn check_ineffective_lt(cx: &LateContext, span: Span, m: u128, c: u128, op: &str) {
    if c.is_power_of_two() && m < c {
        span_lint(
            cx,
            INEFFECTIVE_BIT_MASK,
            span,
            &format!(
                "ineffective bit mask: `x {} {}` compared to `{}`, is the same as x compared directly",
                op,
                m,
                c
            ),
        );
    }
}

fn check_ineffective_gt(cx: &LateContext, span: Span, m: u128, c: u128, op: &str) {
    if (c + 1).is_power_of_two() && m <= c {
        span_lint(
            cx,
            INEFFECTIVE_BIT_MASK,
            span,
            &format!(
                "ineffective bit mask: `x {} {}` compared to `{}`, is the same as x compared directly",
                op,
                m,
                c
            ),
        );
    }
}

fn fetch_int_literal(cx: &LateContext, lit: &Expr) -> Option<u128> {
    match constant(cx, lit)?.0 {
        Constant::Int(n) => Some(n),
        _ => None,
    }
}
