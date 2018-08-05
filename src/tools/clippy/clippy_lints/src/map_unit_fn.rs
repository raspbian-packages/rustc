use rustc::hir;
use rustc::lint::*;
use rustc::ty;
use rustc_errors::{Applicability};
use syntax::codemap::Span;
use utils::{in_macro, iter_input_pats, match_type, method_chain_args, snippet, span_lint_and_then};
use utils::paths;

#[derive(Clone)]
pub struct Pass;

/// **What it does:** Checks for usage of `option.map(f)` where f is a function
/// or closure that returns the unit type.
///
/// **Why is this bad?** Readability, this can be written more clearly with
/// an if let statement
///
/// **Known problems:** None.
///
/// **Example:**
///
/// ```rust
/// let x: Option<&str> = do_stuff();
/// x.map(log_err_msg);
/// x.map(|msg| log_err_msg(format_msg(msg)))
/// ```
///
/// The correct use would be:
///
/// ```rust
/// let x: Option<&str> = do_stuff();
/// if let Some(msg) = x {
///     log_err_msg(msg)
/// }
/// if let Some(msg) = x {
///     log_err_msg(format_msg(msg))
/// }
/// ```
declare_clippy_lint! {
    pub OPTION_MAP_UNIT_FN,
    complexity,
    "using `option.map(f)`, where f is a function or closure that returns ()"
}

/// **What it does:** Checks for usage of `result.map(f)` where f is a function
/// or closure that returns the unit type.
///
/// **Why is this bad?** Readability, this can be written more clearly with
/// an if let statement
///
/// **Known problems:** None.
///
/// **Example:**
///
/// ```rust
/// let x: Result<&str, &str> = do_stuff();
/// x.map(log_err_msg);
/// x.map(|msg| log_err_msg(format_msg(msg)))
/// ```
///
/// The correct use would be:
///
/// ```rust
/// let x: Result<&str, &str> = do_stuff();
/// if let Ok(msg) = x {
///     log_err_msg(msg)
/// }
/// if let Ok(msg) = x {
///     log_err_msg(format_msg(msg))
/// }
/// ```
declare_clippy_lint! {
    pub RESULT_MAP_UNIT_FN,
    complexity,
    "using `result.map(f)`, where f is a function or closure that returns ()"
}


impl LintPass for Pass {
    fn get_lints(&self) -> LintArray {
        lint_array!(OPTION_MAP_UNIT_FN, RESULT_MAP_UNIT_FN)
    }
}

fn is_unit_type(ty: ty::Ty) -> bool {
    match ty.sty {
        ty::TyTuple(slice) => slice.is_empty(),
        ty::TyNever => true,
        _ => false,
    }
}

fn is_unit_function(cx: &LateContext, expr: &hir::Expr) -> bool {
    let ty = cx.tables.expr_ty(expr);

    if let ty::TyFnDef(id, _) = ty.sty {
        if let Some(fn_type) = cx.tcx.fn_sig(id).no_late_bound_regions() {
            return is_unit_type(fn_type.output());
        }
    }
    false
}

fn is_unit_expression(cx: &LateContext, expr: &hir::Expr) -> bool {
    is_unit_type(cx.tables.expr_ty(expr))
}

/// The expression inside a closure may or may not have surrounding braces and
/// semicolons, which causes problems when generating a suggestion. Given an
/// expression that evaluates to '()' or '!', recursively remove useless braces
/// and semi-colons until is suitable for including in the suggestion template
fn reduce_unit_expression<'a>(cx: &LateContext, expr: &'a hir::Expr) -> Option<Span> {
    if !is_unit_expression(cx, expr) {
        return None;
    }

    match expr.node {
        hir::ExprCall(_, _) |
        hir::ExprMethodCall(_, _, _) => {
            // Calls can't be reduced any more
            Some(expr.span)
        },
        hir::ExprBlock(ref block, _) => {
            match (&block.stmts[..], block.expr.as_ref()) {
                (&[], Some(inner_expr)) => {
                    // If block only contains an expression,
                    // reduce `{ X }` to `X`
                    reduce_unit_expression(cx, inner_expr)
                },
                (&[ref inner_stmt], None) => {
                    // If block only contains statements,
                    // reduce `{ X; }` to `X` or `X;`
                    match inner_stmt.node {
                        hir::StmtDecl(ref d, _) => Some(d.span),
                        hir::StmtExpr(ref e, _) => Some(e.span),
                        hir::StmtSemi(_, _) => Some(inner_stmt.span),
                    }
                },
                _ => {
                    // For closures that contain multiple statements
                    // it's difficult to get a correct suggestion span
                    // for all cases (multi-line closures specifically)
                    //
                    // We do not attempt to build a suggestion for those right now.
                    None
                }
            }
        },
        _ => None,
    }
}

fn unit_closure<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, expr: &'a hir::Expr) -> Option<(&'tcx hir::Arg, &'a hir::Expr)> {
    if let hir::ExprClosure(_, ref decl, inner_expr_id, _, _) = expr.node {
        let body = cx.tcx.hir.body(inner_expr_id);
        let body_expr = &body.value;

        if_chain! {
            if decl.inputs.len() == 1;
            if is_unit_expression(cx, body_expr);
            if let Some(binding) = iter_input_pats(&decl, body).next();
            then {
                return Some((binding, body_expr));
            }
        }
    }
    None
}

/// Builds a name for the let binding variable (var_arg)
///
/// `x.field` => `x_field`
/// `y` => `_y`
///
/// Anything else will return `_`.
fn let_binding_name(cx: &LateContext, var_arg: &hir::Expr) -> String {
    match &var_arg.node {
        hir::ExprField(_, _) => snippet(cx, var_arg.span, "_").replace(".", "_"),
        hir::ExprPath(_) => format!("_{}", snippet(cx, var_arg.span, "")),
        _ => "_".to_string()
    }
}

fn suggestion_msg(function_type: &str, map_type: &str) -> String {
    format!(
        "called `map(f)` on an {0} value where `f` is a unit {1}",
        map_type,
        function_type
    )
}

fn lint_map_unit_fn(cx: &LateContext, stmt: &hir::Stmt, expr: &hir::Expr, map_args: &[hir::Expr]) {
    let var_arg = &map_args[0];
    let fn_arg = &map_args[1];

    let (map_type, variant, lint) =
        if match_type(cx, cx.tables.expr_ty(var_arg), &paths::OPTION) {
            ("Option", "Some", OPTION_MAP_UNIT_FN)
        } else if match_type(cx, cx.tables.expr_ty(var_arg), &paths::RESULT) {
            ("Result", "Ok", RESULT_MAP_UNIT_FN)
        } else {
            return
        };

    if is_unit_function(cx, fn_arg) {
        let msg = suggestion_msg("function", map_type);
        let suggestion = format!("if let {0}({1}) = {2} {{ {3}(...) }}",
                                 variant,
                                 let_binding_name(cx, var_arg),
                                 snippet(cx, var_arg.span, "_"),
                                 snippet(cx, fn_arg.span, "_"));

        span_lint_and_then(cx, lint, expr.span, &msg, |db| {
            db.span_suggestion_with_applicability(stmt.span,
                                                  "try this",
                                                  suggestion,
                                                  Applicability::Unspecified);
        });
    } else if let Some((binding, closure_expr)) = unit_closure(cx, fn_arg) {
        let msg = suggestion_msg("closure", map_type);

        span_lint_and_then(cx, lint, expr.span, &msg, |db| {
            if let Some(reduced_expr_span) = reduce_unit_expression(cx, closure_expr) {
                let suggestion = format!("if let {0}({1}) = {2} {{ {3} }}",
                                         variant,
                                         snippet(cx, binding.pat.span, "_"),
                                         snippet(cx, var_arg.span, "_"),
                                         snippet(cx, reduced_expr_span, "_"));
                db.span_suggestion(stmt.span, "try this", suggestion);
            } else {
                let suggestion = format!("if let {0}({1}) = {2} {{ ... }}",
                                         variant,
                                         snippet(cx, binding.pat.span, "_"),
                                         snippet(cx, var_arg.span, "_"));
                db.span_suggestion_with_applicability(stmt.span,
                                                      "try this",
                                                      suggestion,
                                                      Applicability::Unspecified);
            }
        });
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for Pass {
    fn check_stmt(&mut self, cx: &LateContext, stmt: &hir::Stmt) {
        if in_macro(stmt.span) {
            return;
        }

        if let hir::StmtSemi(ref expr, _) = stmt.node {
            if let hir::ExprMethodCall(_, _, _) = expr.node {
                if let Some(arglists) = method_chain_args(expr, &["map"]) {
                    lint_map_unit_fn(cx, stmt, expr, arglists[0]);
                }
            }
        }
    }
}
