use rustc::lint::*;
use rustc::hir::*;
use std::f64::consts as f64;
use syntax::ast::{Lit, LitKind, FloatTy};
use syntax::symbol;
use utils::span_lint;

/// **What it does:** Checks for floating point literals that approximate
/// constants which are defined in
/// [`std::f32::consts`](https://doc.rust-lang.
/// org/stable/std/f32/consts/#constants)
/// or
/// [`std::f64::consts`](https://doc.rust-lang.
/// org/stable/std/f64/consts/#constants),
/// respectively, suggesting to use the predefined constant.
///
/// **Why is this bad?** Usually, the definition in the standard library is more
/// precise than what people come up with. If you find that your definition is
/// actually more precise, please [file a Rust
/// issue](https://github.com/rust-lang/rust/issues).
///
/// **Known problems:** If you happen to have a value that is within 1/8192 of a
/// known constant, but is not *and should not* be the same, this lint will
/// report your value anyway. We have not yet noticed any false positives in
/// code we tested clippy with (this includes servo), but YMMV.
///
/// **Example:**
/// ```rust
/// let x = 3.14;
/// ```
declare_lint! {
    pub APPROX_CONSTANT,
    Warn,
    "the approximate of a known float constant (in `std::fXX::consts`)"
}

// Tuples are of the form (constant, name, min_digits)
const KNOWN_CONSTS: &'static [(f64, &'static str, usize)] = &[
    (f64::E, "E", 4),
    (f64::FRAC_1_PI, "FRAC_1_PI", 4),
    (f64::FRAC_1_SQRT_2, "FRAC_1_SQRT_2", 5),
    (f64::FRAC_2_PI, "FRAC_2_PI", 5),
    (f64::FRAC_2_SQRT_PI, "FRAC_2_SQRT_PI", 5),
    (f64::FRAC_PI_2, "FRAC_PI_2", 5),
    (f64::FRAC_PI_3, "FRAC_PI_3", 5),
    (f64::FRAC_PI_4, "FRAC_PI_4", 5),
    (f64::FRAC_PI_6, "FRAC_PI_6", 5),
    (f64::FRAC_PI_8, "FRAC_PI_8", 5),
    (f64::LN_10, "LN_10", 5),
    (f64::LN_2, "LN_2", 5),
    (f64::LOG10_E, "LOG10_E", 5),
    (f64::LOG2_E, "LOG2_E", 5),
    (f64::PI, "PI", 3),
    (f64::SQRT_2, "SQRT_2", 5),
];

#[derive(Copy, Clone)]
pub struct Pass;

impl LintPass for Pass {
    fn get_lints(&self) -> LintArray {
        lint_array!(APPROX_CONSTANT)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for Pass {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, e: &'tcx Expr) {
        if let ExprLit(ref lit) = e.node {
            check_lit(cx, lit, e);
        }
    }
}

fn check_lit(cx: &LateContext, lit: &Lit, e: &Expr) {
    match lit.node {
        LitKind::Float(ref s, FloatTy::F32) => check_known_consts(cx, e, s, "f32"),
        LitKind::Float(ref s, FloatTy::F64) => check_known_consts(cx, e, s, "f64"),
        LitKind::FloatUnsuffixed(ref s) => check_known_consts(cx, e, s, "f{32, 64}"),
        _ => (),
    }
}

fn check_known_consts(cx: &LateContext, e: &Expr, s: &symbol::Symbol, module: &str) {
    let s = s.as_str();
    if s.parse::<f64>().is_ok() {
        for &(constant, name, min_digits) in KNOWN_CONSTS {
            if is_approx_const(constant, &s, min_digits) {
                span_lint(
                    cx,
                    APPROX_CONSTANT,
                    e.span,
                    &format!(
                        "approximate value of `{}::consts::{}` found. \
                                    Consider using it directly",
                        module,
                        &name
                    ),
                );
                return;
            }
        }
    }
}

/// Returns false if the number of significant figures in `value` are
/// less than `min_digits`; otherwise, returns true if `value` is equal
/// to `constant`, rounded to the number of digits present in `value`.
fn is_approx_const(constant: f64, value: &str, min_digits: usize) -> bool {
    if value.len() <= min_digits {
        false
    } else {
        let round_const = format!("{:.*}", value.len() - 2, constant);

        let mut trunc_const = constant.to_string();
        if trunc_const.len() > value.len() {
            trunc_const.truncate(value.len());
        }

        (value == round_const) || (value == trunc_const)
    }
}
