use rustc::lint::*;
use rustc::hir::*;
use utils::{span_lint, get_trait_def_id, paths};

/// **What it does:** Checks for mis-uses of the serde API.
///
/// **Why is this bad?** Serde is very finnicky about how its API should be
/// used, but the type system can't be used to enforce it (yet).
///
/// **Known problems:** None.
///
/// **Example:** Implementing `Visitor::visit_string` but not
/// `Visitor::visit_str`.
declare_lint! {
    pub SERDE_API_MISUSE,
    Warn,
    "various things that will negatively affect your serde experience"
}


#[derive(Copy, Clone)]
pub struct Serde;

impl LintPass for Serde {
    fn get_lints(&self) -> LintArray {
        lint_array!(SERDE_API_MISUSE)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for Serde {
    fn check_item(&mut self, cx: &LateContext<'a, 'tcx>, item: &'tcx Item) {
        if let ItemImpl(_, _, _, _, Some(ref trait_ref), _, ref items) = item.node {
            let did = trait_ref.path.def.def_id();
            if let Some(visit_did) = get_trait_def_id(cx, &paths::SERDE_DE_VISITOR) {
                if did == visit_did {
                    let mut seen_str = None;
                    let mut seen_string = None;
                    for item in items {
                        match &*item.name.as_str() {
                            "visit_str" => seen_str = Some(item.span),
                            "visit_string" => seen_string = Some(item.span),
                            _ => {},
                        }
                    }
                    if let Some(span) = seen_string {
                        if seen_str.is_none() {
                            span_lint(
                                cx,
                                SERDE_API_MISUSE,
                                span,
                                "you should not implement `visit_string` without also implementing `visit_str`",
                            );
                        }
                    }
                }
            }
        }
    }
}
