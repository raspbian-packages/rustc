//! checks for `#[inline]` on trait methods without bodies

use rustc::lint::*;
use rustc::hir::*;
use syntax::ast::{Attribute, Name};
use utils::span_lint_and_then;
use utils::sugg::DiagnosticBuilderExt;

/// **What it does:** Checks for `#[inline]` on trait methods without bodies
///
/// **Why is this bad?** Only implementations of trait methods may be inlined.
/// The inline attribute is ignored for trait methods without bodies.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// trait Animal {
///     #[inline]
///     fn name(&self) -> &'static str;
/// }
/// ```
declare_clippy_lint! {
    pub INLINE_FN_WITHOUT_BODY,
    correctness,
    "use of `#[inline]` on trait methods without bodies"
}

#[derive(Copy, Clone)]
pub struct Pass;

impl LintPass for Pass {
    fn get_lints(&self) -> LintArray {
        lint_array!(INLINE_FN_WITHOUT_BODY)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for Pass {
    fn check_trait_item(&mut self, cx: &LateContext<'a, 'tcx>, item: &'tcx TraitItem) {
        if let TraitItemKind::Method(_, TraitMethod::Required(_)) = item.node {
            check_attrs(cx, &item.name, &item.attrs);
        }
    }
}

fn check_attrs(cx: &LateContext, name: &Name, attrs: &[Attribute]) {
    for attr in attrs {
        if attr.name().map_or(true, |n| n != "inline") {
            continue;
        }

        span_lint_and_then(
            cx,
            INLINE_FN_WITHOUT_BODY,
            attr.span,
            &format!("use of `#[inline]` on trait method `{}` which has no body", name),
            |db| {
                db.suggest_remove_item(cx, attr.span, "remove");
            },
        );
    }
}
