//! Checks for uses of mutex where an atomic value could be used
//!
//! This lint is **warn** by default

use rustc::lint::{LintPass, LintArray, LateLintPass, LateContext};
use rustc::ty::{self, Ty};
use rustc::hir::Expr;
use syntax::ast;
use utils::{match_type, paths, span_lint};

/// **What it does:** Checks for usages of `Mutex<X>` where an atomic will do.
///
/// **Why is this bad?** Using a mutex just to make access to a plain bool or
/// reference sequential is shooting flies with cannons.
/// `std::atomic::AtomicBool` and `std::atomic::AtomicPtr` are leaner and
/// faster.
///
/// **Known problems:** This lint cannot detect if the mutex is actually used
/// for waiting before a critical section.
///
/// **Example:**
/// ```rust
/// let x = Mutex::new(&y);
/// ```
declare_lint! {
    pub MUTEX_ATOMIC,
    Warn,
    "using a mutex where an atomic value could be used instead"
}

/// **What it does:** Checks for usages of `Mutex<X>` where `X` is an integral
/// type.
///
/// **Why is this bad?** Using a mutex just to make access to a plain integer
/// sequential is
/// shooting flies with cannons. `std::atomic::usize` is leaner and faster.
///
/// **Known problems:** This lint cannot detect if the mutex is actually used
/// for waiting before a critical section.
///
/// **Example:**
/// ```rust
/// let x = Mutex::new(0usize);
/// ```
declare_lint! {
    pub MUTEX_INTEGER,
    Allow,
    "using a mutex for an integer type"
}

impl LintPass for MutexAtomic {
    fn get_lints(&self) -> LintArray {
        lint_array!(MUTEX_ATOMIC, MUTEX_INTEGER)
    }
}

pub struct MutexAtomic;

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for MutexAtomic {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, expr: &'tcx Expr) {
        let ty = cx.tables.expr_ty(expr);
        if let ty::TyAdt(_, subst) = ty.sty {
            if match_type(cx, ty, &paths::MUTEX) {
                let mutex_param = subst.type_at(0);
                if let Some(atomic_name) = get_atomic_name(mutex_param) {
                    let msg = format!(
                        "Consider using an {} instead of a Mutex here. If you just want the locking \
                                       behaviour and not the internal type, consider using Mutex<()>.",
                        atomic_name
                    );
                    match mutex_param.sty {
                        ty::TyUint(t) if t != ast::UintTy::Us => span_lint(cx, MUTEX_INTEGER, expr.span, &msg),
                        ty::TyInt(t) if t != ast::IntTy::Is => span_lint(cx, MUTEX_INTEGER, expr.span, &msg),
                        _ => span_lint(cx, MUTEX_ATOMIC, expr.span, &msg),
                    };
                }
            }
        }
    }
}

fn get_atomic_name(ty: Ty) -> Option<(&'static str)> {
    match ty.sty {
        ty::TyBool => Some("AtomicBool"),
        ty::TyUint(_) => Some("AtomicUsize"),
        ty::TyInt(_) => Some("AtomicIsize"),
        ty::TyRawPtr(_) => Some("AtomicPtr"),
        _ => None,
    }
}
