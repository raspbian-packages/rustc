//! lint on `use`ing all variants of an enum

use rustc::hir::*;
use rustc::hir::def::Def;
use rustc::lint::{LateContext, LateLintPass, LintArray, LintPass};
use syntax::ast::NodeId;
use syntax::codemap::Span;
use utils::span_lint;

/// **What it does:** Checks for `use Enum::*`.
///
/// **Why is this bad?** It is usually better style to use the prefixed name of
/// an enumeration variant, rather than importing variants.
///
/// **Known problems:** Old-style enumerations that prefix the variants are
/// still around.
///
/// **Example:**
/// ```rust
/// use std::cmp::Ordering::*;
/// ```
declare_clippy_lint! {
    pub ENUM_GLOB_USE,
    pedantic,
    "use items that import all variants of an enum"
}

pub struct EnumGlobUse;

impl LintPass for EnumGlobUse {
    fn get_lints(&self) -> LintArray {
        lint_array!(ENUM_GLOB_USE)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for EnumGlobUse {
    fn check_mod(&mut self, cx: &LateContext<'a, 'tcx>, m: &'tcx Mod, _: Span, _: NodeId) {
        // only check top level `use` statements
        for item in &m.item_ids {
            self.lint_item(cx, cx.tcx.hir.expect_item(item.id));
        }
    }
}

impl EnumGlobUse {
    fn lint_item(&self, cx: &LateContext, item: &Item) {
        if item.vis == Visibility::Public {
            return; // re-exports are fine
        }
        if let ItemUse(ref path, UseKind::Glob) = item.node {
            if let Def::Enum(_) = path.def {
                span_lint(
                    cx,
                    ENUM_GLOB_USE,
                    item.span,
                    "don't use glob imports for enum variants",
                );
            }
        }
    }
}
