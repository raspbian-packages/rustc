//! lint on multiple versions of a crate being used

use rustc::lint::*;
use syntax::ast::*;

use cargo_metadata;
use itertools::Itertools;

/// **What it does:** Checks to see if multiple versions of a crate are being
/// used.
///
/// **Why is this bad?** This bloats the size of targets, and can lead to
/// confusing error messages when structs or traits are used interchangeably
/// between different versions of a crate.
///
/// **Known problems:** Because this can be caused purely by the dependencies
/// themselves, it's not always possible to fix this issue.
///
/// **Example:**
/// ```toml
/// # This will pull in both winapi v0.3.4 and v0.2.8, triggering a warning.
/// [dependencies]
/// ctrlc = "3.1.0"
/// ansi_term = "0.11.0"
/// ```
declare_clippy_lint! {
    pub MULTIPLE_CRATE_VERSIONS,
    cargo,
    "multiple versions of the same crate being used"
}

pub struct Pass;

impl LintPass for Pass {
    fn get_lints(&self) -> LintArray {
        lint_array!(MULTIPLE_CRATE_VERSIONS)
    }
}

impl EarlyLintPass for Pass {
    fn check_crate(&mut self, cx: &EarlyContext, krate: &Crate) {
        let metadata = match cargo_metadata::metadata_deps(None, true) {
            Ok(metadata) => metadata,
            Err(_) => {
                cx.span_lint(
                    MULTIPLE_CRATE_VERSIONS,
                    krate.span,
                    "could not read cargo metadata"
                );

                return;
            }
        };

        let mut packages = metadata.packages;
        packages.sort_by(|a, b| a.name.cmp(&b.name));

        for (name, group) in &packages.into_iter().group_by(|p| p.name.clone()) {
            let group: Vec<cargo_metadata::Package> = group.collect();

            if group.len() > 1 {
                let versions = group.into_iter().map(|p| p.version).join(", ");

                cx.span_lint(
                    MULTIPLE_CRATE_VERSIONS,
                    krate.span,
                    &format!("multiple versions for dependency `{}`: {}", name, versions),
                );
            }
        }
    }
}
