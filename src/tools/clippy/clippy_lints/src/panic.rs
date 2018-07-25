use rustc::hir::*;
use rustc::lint::*;
use syntax::ast::LitKind;
use utils::{is_direct_expn_of, match_def_path, opt_def_id, paths, resolve_node, span_lint};

/// **What it does:** Checks for missing parameters in `panic!`.
///
/// **Why is this bad?** Contrary to the `format!` family of macros, there are
/// two forms of `panic!`: if there are no parameters given, the first argument
/// is not a format string and used literally. So while `format!("{}")` will
/// fail to compile, `panic!("{}")` will not.
///
/// **Known problems:** Should you want to use curly brackets in `panic!`
/// without any parameter, this lint will warn.
///
/// **Example:**
/// ```rust
/// panic!("This `panic!` is probably missing a parameter there: {}");
/// ```
declare_clippy_lint! {
    pub PANIC_PARAMS,
    style,
    "missing parameters in `panic!` calls"
}

#[allow(missing_copy_implementations)]
pub struct Pass;

impl LintPass for Pass {
    fn get_lints(&self) -> LintArray {
        lint_array!(PANIC_PARAMS)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for Pass {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, expr: &'tcx Expr) {
        if_chain! {
            if let ExprBlock(ref block) = expr.node;
            if let Some(ref ex) = block.expr;
            if let ExprCall(ref fun, ref params) = ex.node;
            if params.len() == 2;
            if let ExprPath(ref qpath) = fun.node;
            if let Some(fun_def_id) = opt_def_id(resolve_node(cx, qpath, fun.hir_id));
            if match_def_path(cx.tcx, fun_def_id, &paths::BEGIN_PANIC);
            if let ExprLit(ref lit) = params[0].node;
            if is_direct_expn_of(expr.span, "panic").is_some();
            if let LitKind::Str(ref string, _) = lit.node;
            if let Some(par) = string.as_str().find('{');
            if string.as_str()[par..].contains('}');
            if params[0].span.source_callee().is_none();
            if params[0].span.lo() != params[0].span.hi();
            then {
                span_lint(cx, PANIC_PARAMS, params[0].span,
                          "you probably are missing some parameter in your format string");
            }
        }
    }
}
