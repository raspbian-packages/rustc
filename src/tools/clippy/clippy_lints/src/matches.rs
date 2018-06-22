use rustc::hir::*;
use rustc::lint::*;
use rustc::ty::{self, Ty};
use std::cmp::Ordering;
use std::collections::Bound;
use syntax::ast::LitKind;
use syntax::codemap::Span;
use utils::paths;
use utils::{expr_block, in_external_macro, is_allowed, is_expn_of, match_qpath, match_type, multispan_sugg,
            remove_blocks, snippet, span_lint_and_sugg, span_lint_and_then, span_note_and_lint, walk_ptrs_ty};
use utils::sugg::Sugg;
use consts::{constant, Constant};

/// **What it does:** Checks for matches with a single arm where an `if let`
/// will usually suffice.
///
/// **Why is this bad?** Just readability – `if let` nests less than a `match`.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// match x {
///     Some(ref foo) => bar(foo),
///     _ => ()
/// }
/// ```
declare_clippy_lint! {
    pub SINGLE_MATCH,
    style,
    "a match statement with a single nontrivial arm (i.e. where the other arm \
     is `_ => {}`) instead of `if let`"
}

/// **What it does:** Checks for matches with a two arms where an `if let` will
/// usually suffice.
///
/// **Why is this bad?** Just readability – `if let` nests less than a `match`.
///
/// **Known problems:** Personal style preferences may differ.
///
/// **Example:**
/// ```rust
/// match x {
///     Some(ref foo) => bar(foo),
///     _ => bar(other_ref),
/// }
/// ```
declare_clippy_lint! {
    pub SINGLE_MATCH_ELSE,
    pedantic,
    "a match statement with a two arms where the second arm's pattern is a wildcard \
     instead of `if let`"
}

/// **What it does:** Checks for matches where all arms match a reference,
/// suggesting to remove the reference and deref the matched expression
/// instead. It also checks for `if let &foo = bar` blocks.
///
/// **Why is this bad?** It just makes the code less readable. That reference
/// destructuring adds nothing to the code.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// match x {
///     &A(ref y) => foo(y),
///     &B => bar(),
///     _ => frob(&x),
/// }
/// ```
declare_clippy_lint! {
    pub MATCH_REF_PATS,
    style,
    "a match or `if let` with all arms prefixed with `&` instead of deref-ing the match expression"
}

/// **What it does:** Checks for matches where match expression is a `bool`. It
/// suggests to replace the expression with an `if...else` block.
///
/// **Why is this bad?** It makes the code less readable.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// let condition: bool = true;
/// match condition {
///     true => foo(),
///     false => bar(),
/// }
/// ```
declare_clippy_lint! {
    pub MATCH_BOOL,
    style,
    "a match on a boolean expression instead of an `if..else` block"
}

/// **What it does:** Checks for overlapping match arms.
///
/// **Why is this bad?** It is likely to be an error and if not, makes the code
/// less obvious.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// let x = 5;
/// match x {
///     1 ... 10 => println!("1 ... 10"),
///     5 ... 15 => println!("5 ... 15"),
///     _ => (),
/// }
/// ```
declare_clippy_lint! {
    pub MATCH_OVERLAPPING_ARM,
    style,
    "a match with overlapping arms"
}

/// **What it does:** Checks for arm which matches all errors with `Err(_)`
/// and take drastic actions like `panic!`.
///
/// **Why is this bad?** It is generally a bad practice, just like
/// catching all exceptions in java with `catch(Exception)`
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// let x : Result(i32, &str) = Ok(3);
/// match x {
///     Ok(_) => println!("ok"),
///     Err(_) => panic!("err"),
/// }
/// ```
declare_clippy_lint! {
    pub MATCH_WILD_ERR_ARM,
    style,
    "a match with `Err(_)` arm and take drastic actions"
}

/// **What it does:** Checks for match which is used to add a reference to an
/// `Option` value.
///
/// **Why is this bad?** Using `as_ref()` or `as_mut()` instead is shorter.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// let x: Option<()> = None;
/// let r: Option<&()> = match x {
///   None => None,
///   Some(ref v) => Some(v),
/// };
/// ```
declare_clippy_lint! {
    pub MATCH_AS_REF,
    complexity,
    "a match on an Option value instead of using `as_ref()` or `as_mut`"
}

#[allow(missing_copy_implementations)]
pub struct MatchPass;

impl LintPass for MatchPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(
            SINGLE_MATCH,
            MATCH_REF_PATS,
            MATCH_BOOL,
            SINGLE_MATCH_ELSE,
            MATCH_OVERLAPPING_ARM,
            MATCH_WILD_ERR_ARM,
            MATCH_AS_REF
        )
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for MatchPass {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, expr: &'tcx Expr) {
        if in_external_macro(cx, expr.span) {
            return;
        }
        if let ExprMatch(ref ex, ref arms, MatchSource::Normal) = expr.node {
            check_single_match(cx, ex, arms, expr);
            check_match_bool(cx, ex, arms, expr);
            check_overlapping_arms(cx, ex, arms);
            check_wild_err_arm(cx, ex, arms);
            check_match_as_ref(cx, ex, arms, expr);
        }
        if let ExprMatch(ref ex, ref arms, _) = expr.node {
            check_match_ref_pats(cx, ex, arms, expr);
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn check_single_match(cx: &LateContext, ex: &Expr, arms: &[Arm], expr: &Expr) {
    if arms.len() == 2 &&
      arms[0].pats.len() == 1 && arms[0].guard.is_none() &&
      arms[1].pats.len() == 1 && arms[1].guard.is_none() {
        let els = remove_blocks(&arms[1].body);
        let els = if is_unit_expr(els) {
            None
        } else if let ExprBlock(_) = els.node {
            // matches with blocks that contain statements are prettier as `if let + else`
            Some(els)
        } else {
            // allow match arms with just expressions
            return;
        };
        let ty = cx.tables.expr_ty(ex);
        if ty.sty != ty::TyBool || is_allowed(cx, MATCH_BOOL, ex.id) {
            check_single_match_single_pattern(cx, ex, arms, expr, els);
            check_single_match_opt_like(cx, ex, arms, expr, ty, els);
        }
    }
}

fn check_single_match_single_pattern(cx: &LateContext, ex: &Expr, arms: &[Arm], expr: &Expr, els: Option<&Expr>) {
    if arms[1].pats[0].node == PatKind::Wild {
        report_single_match_single_pattern(cx, ex, arms, expr, els);
    }
}

fn report_single_match_single_pattern(cx: &LateContext, ex: &Expr, arms: &[Arm], expr: &Expr, els: Option<&Expr>) {
    let lint = if els.is_some() {
        SINGLE_MATCH_ELSE
    } else {
        SINGLE_MATCH
    };
    let els_str = els.map_or(String::new(), |els| format!(" else {}", expr_block(cx, els, None, "..")));
    span_lint_and_sugg(
        cx,
        lint,
        expr.span,
        "you seem to be trying to use match for destructuring a single pattern. Consider using `if \
         let`",
        "try this",
        format!(
            "if let {} = {} {}{}",
            snippet(cx, arms[0].pats[0].span, ".."),
            snippet(cx, ex.span, ".."),
            expr_block(cx, &arms[0].body, None, ".."),
            els_str
        ),
    );
}

fn check_single_match_opt_like(cx: &LateContext, ex: &Expr, arms: &[Arm], expr: &Expr, ty: Ty, els: Option<&Expr>) {
    // list of candidate Enums we know will never get any more members
    let candidates = &[
        (&paths::COW, "Borrowed"),
        (&paths::COW, "Cow::Borrowed"),
        (&paths::COW, "Cow::Owned"),
        (&paths::COW, "Owned"),
        (&paths::OPTION, "None"),
        (&paths::RESULT, "Err"),
        (&paths::RESULT, "Ok"),
    ];

    let path = match arms[1].pats[0].node {
        PatKind::TupleStruct(ref path, ref inner, _) => {
            // contains any non wildcard patterns? e.g. Err(err)
            if inner.iter().any(|pat| pat.node != PatKind::Wild) {
                return;
            }
            print::to_string(print::NO_ANN, |s| s.print_qpath(path, false))
        },
        PatKind::Binding(BindingAnnotation::Unannotated, _, ident, None) => ident.node.to_string(),
        PatKind::Path(ref path) => print::to_string(print::NO_ANN, |s| s.print_qpath(path, false)),
        _ => return,
    };

    for &(ty_path, pat_path) in candidates {
        if path == *pat_path && match_type(cx, ty, ty_path) {
            report_single_match_single_pattern(cx, ex, arms, expr, els);
        }
    }
}

fn check_match_bool(cx: &LateContext, ex: &Expr, arms: &[Arm], expr: &Expr) {
    // type of expression == bool
    if cx.tables.expr_ty(ex).sty == ty::TyBool {
        span_lint_and_then(
            cx,
            MATCH_BOOL,
            expr.span,
            "you seem to be trying to match on a boolean expression",
            move |db| {
                if arms.len() == 2 && arms[0].pats.len() == 1 {
                    // no guards
                    let exprs = if let PatKind::Lit(ref arm_bool) = arms[0].pats[0].node {
                        if let ExprLit(ref lit) = arm_bool.node {
                            match lit.node {
                                LitKind::Bool(true) => Some((&*arms[0].body, &*arms[1].body)),
                                LitKind::Bool(false) => Some((&*arms[1].body, &*arms[0].body)),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some((true_expr, false_expr)) = exprs {
                        let sugg = match (is_unit_expr(true_expr), is_unit_expr(false_expr)) {
                            (false, false) => Some(format!(
                                "if {} {} else {}",
                                snippet(cx, ex.span, "b"),
                                expr_block(cx, true_expr, None, ".."),
                                expr_block(cx, false_expr, None, "..")
                            )),
                            (false, true) => Some(format!(
                                "if {} {}",
                                snippet(cx, ex.span, "b"),
                                expr_block(cx, true_expr, None, "..")
                            )),
                            (true, false) => {
                                let test = Sugg::hir(cx, ex, "..");
                                Some(format!("if {} {}", !test, expr_block(cx, false_expr, None, "..")))
                            },
                            (true, true) => None,
                        };

                        if let Some(sugg) = sugg {
                            db.span_suggestion(expr.span, "consider using an if/else expression", sugg);
                        }
                    }
                }
            },
        );
    }
}

fn check_overlapping_arms<'a, 'tcx>(cx: &LateContext<'a, 'tcx>, ex: &'tcx Expr, arms: &'tcx [Arm]) {
    if arms.len() >= 2 && cx.tables.expr_ty(ex).is_integral() {
        let ranges = all_ranges(cx, arms);
        let type_ranges = type_ranges(&ranges);
        if !type_ranges.is_empty() {
            if let Some((start, end)) = overlapping(&type_ranges) {
                span_note_and_lint(
                    cx,
                    MATCH_OVERLAPPING_ARM,
                    start.span,
                    "some ranges overlap",
                    end.span,
                    "overlaps with this",
                );
            }
        }
    }
}

fn check_wild_err_arm(cx: &LateContext, ex: &Expr, arms: &[Arm]) {
    let ex_ty = walk_ptrs_ty(cx.tables.expr_ty(ex));
    if match_type(cx, ex_ty, &paths::RESULT) {
        for arm in arms {
            if let PatKind::TupleStruct(ref path, ref inner, _) = arm.pats[0].node {
                let path_str = print::to_string(print::NO_ANN, |s| s.print_qpath(path, false));
                if_chain! {
                    if path_str == "Err";
                    if inner.iter().any(|pat| pat.node == PatKind::Wild);
                    if let ExprBlock(ref block) = arm.body.node;
                    if is_panic_block(block);
                    then {
                        // `Err(_)` arm with `panic!` found
                        span_note_and_lint(cx,
                                           MATCH_WILD_ERR_ARM,
                                           arm.pats[0].span,
                                           "Err(_) will match all errors, maybe not a good idea",
                                           arm.pats[0].span,
                                           "to remove this warning, match each error seperately \
                                            or use unreachable macro");
                    }
                }
            }
        }
    }
}

// If the block contains only a `panic!` macro (as expression or statement)
fn is_panic_block(block: &Block) -> bool {
    match (&block.expr, block.stmts.len(), block.stmts.first()) {
        (&Some(ref exp), 0, _) => {
            is_expn_of(exp.span, "panic").is_some() && is_expn_of(exp.span, "unreachable").is_none()
        },
        (&None, 1, Some(stmt)) => {
            is_expn_of(stmt.span, "panic").is_some() && is_expn_of(stmt.span, "unreachable").is_none()
        },
        _ => false,
    }
}

fn check_match_ref_pats(cx: &LateContext, ex: &Expr, arms: &[Arm], expr: &Expr) {
    if has_only_ref_pats(arms) {
        let mut suggs = Vec::new();
        let (title, msg) = if let ExprAddrOf(Mutability::MutImmutable, ref inner) = ex.node {
            suggs.push((ex.span, Sugg::hir(cx, inner, "..").to_string()));
            (
                "you don't need to add `&` to both the expression and the patterns",
                "try",
            )
        } else {
            suggs.push((ex.span, Sugg::hir(cx, ex, "..").deref().to_string()));
            (
                "you don't need to add `&` to all patterns",
                "instead of prefixing all patterns with `&`, you can dereference the expression",
            )
        };

        suggs.extend(arms.iter().flat_map(|a| &a.pats).filter_map(|p| {
            if let PatKind::Ref(ref refp, _) = p.node {
                Some((p.span, snippet(cx, refp.span, "..").to_string()))
            } else {
                None
            }
        }));

        span_lint_and_then(cx, MATCH_REF_PATS, expr.span, title, |db| {
            multispan_sugg(db, msg.to_owned(), suggs);
        });
    }
}

fn check_match_as_ref(cx: &LateContext, ex: &Expr, arms: &[Arm], expr: &Expr) {
    if arms.len() == 2 &&
        arms[0].pats.len() == 1 && arms[0].guard.is_none() &&
        arms[1].pats.len() == 1 && arms[1].guard.is_none() {
        let arm_ref: Option<BindingAnnotation> = if is_none_arm(&arms[0]) {
            is_ref_some_arm(&arms[1])
        } else if is_none_arm(&arms[1]) {
            is_ref_some_arm(&arms[0])
        } else {
            None
        };
        if let Some(rb) = arm_ref {
            let suggestion = if rb == BindingAnnotation::Ref { "as_ref" } else { "as_mut" };
            span_lint_and_sugg(
                cx,
                MATCH_AS_REF,
                expr.span,
                &format!("use {}() instead", suggestion),
                "try this",
                format!("{}.{}()", snippet(cx, ex.span, "_"), suggestion)
            )
        }
    }
}

/// Get all arms that are unbounded `PatRange`s.
fn all_ranges<'a, 'tcx>(
    cx: &LateContext<'a, 'tcx>,
    arms: &'tcx [Arm],
) -> Vec<SpannedRange<Constant>> {
    arms.iter()
        .flat_map(|arm| {
            if let Arm {
                ref pats,
                guard: None,
                ..
            } = *arm
            {
                pats.iter()
            } else {
                [].iter()
            }.filter_map(|pat| {
                if let PatKind::Range(ref lhs, ref rhs, ref range_end) = pat.node {
                    let lhs = constant(cx, lhs)?.0;
                    let rhs = constant(cx, rhs)?.0;
                    let rhs = match *range_end {
                        RangeEnd::Included => Bound::Included(rhs),
                        RangeEnd::Excluded => Bound::Excluded(rhs),
                    };
                    return Some(SpannedRange { span: pat.span, node: (lhs, rhs) });
                }

                if let PatKind::Lit(ref value) = pat.node {
                    let value = constant(cx, value)?.0;
                    return Some(SpannedRange { span: pat.span, node: (value.clone(), Bound::Included(value)) });
                }

                None
            })
        })
        .collect()
}

#[derive(Debug, Eq, PartialEq)]
pub struct SpannedRange<T> {
    pub span: Span,
    pub node: (T, Bound<T>),
}

type TypedRanges = Vec<SpannedRange<u128>>;

/// Get all `Int` ranges or all `Uint` ranges. Mixed types are an error anyway
/// and other types than
/// `Uint` and `Int` probably don't make sense.
fn type_ranges(ranges: &[SpannedRange<Constant>]) -> TypedRanges {
    ranges
        .iter()
        .filter_map(|range| match range.node {
            (
                Constant::Int(start),
                Bound::Included(Constant::Int(end)),
            ) => Some(SpannedRange {
                span: range.span,
                node: (start, Bound::Included(end)),
            }),
            (
                Constant::Int(start),
                Bound::Excluded(Constant::Int(end)),
            ) => Some(SpannedRange {
                span: range.span,
                node: (start, Bound::Excluded(end)),
            }),
            (
                Constant::Int(start),
                Bound::Unbounded,
            ) => Some(SpannedRange {
                span: range.span,
                node: (start, Bound::Unbounded),
            }),
            _ => None,
        })
        .collect()
}

fn is_unit_expr(expr: &Expr) -> bool {
    match expr.node {
        ExprTup(ref v) if v.is_empty() => true,
        ExprBlock(ref b) if b.stmts.is_empty() && b.expr.is_none() => true,
        _ => false,
    }
}

// Checks if arm has the form `None => None`
fn is_none_arm(arm: &Arm) -> bool {
    match arm.pats[0].node {
        PatKind::Path(ref path) if match_qpath(path, &paths::OPTION_NONE) => true,
        _ => false,
    }
}

// Checks if arm has the form `Some(ref v) => Some(v)` (checks for `ref` and `ref mut`)
fn is_ref_some_arm(arm: &Arm) -> Option<BindingAnnotation> {
    if_chain! {
        if let PatKind::TupleStruct(ref path, ref pats, _) = arm.pats[0].node;
        if pats.len() == 1 && match_qpath(path, &paths::OPTION_SOME);
        if let PatKind::Binding(rb, _, ref ident, _) = pats[0].node;
        if rb == BindingAnnotation::Ref || rb == BindingAnnotation::RefMut;
        if let ExprCall(ref e, ref args) = remove_blocks(&arm.body).node;
        if let ExprPath(ref some_path) = e.node;
        if match_qpath(some_path, &paths::OPTION_SOME) && args.len() == 1;
        if let ExprPath(ref qpath) = args[0].node;
        if let &QPath::Resolved(_, ref path2) = qpath;
        if path2.segments.len() == 1 && ident.node == path2.segments[0].name;
        then {
            return Some(rb)
        }
    }
    None
}

fn has_only_ref_pats(arms: &[Arm]) -> bool {
    let mapped = arms.iter()
        .flat_map(|a| &a.pats)
        .map(|p| {
            match p.node {
                PatKind::Ref(..) => Some(true), // &-patterns
                PatKind::Wild => Some(false),   // an "anything" wildcard is also fine
                _ => None,                      // any other pattern is not fine
            }
        })
        .collect::<Option<Vec<bool>>>();
    // look for Some(v) where there's at least one true element
    mapped.map_or(false, |v| v.iter().any(|el| *el))
}

pub fn overlapping<T>(ranges: &[SpannedRange<T>]) -> Option<(&SpannedRange<T>, &SpannedRange<T>)>
where
    T: Copy + Ord,
{
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum Kind<'a, T: 'a> {
        Start(T, &'a SpannedRange<T>),
        End(Bound<T>, &'a SpannedRange<T>),
    }

    impl<'a, T: Copy> Kind<'a, T> {
        fn range(&self) -> &'a SpannedRange<T> {
            match *self {
                Kind::Start(_, r) | Kind::End(_, r) => r,
            }
        }

        fn value(self) -> Bound<T> {
            match self {
                Kind::Start(t, _) => Bound::Included(t),
                Kind::End(t, _) => t,
            }
        }
    }

    impl<'a, T: Copy + Ord> PartialOrd for Kind<'a, T> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl<'a, T: Copy + Ord> Ord for Kind<'a, T> {
        fn cmp(&self, other: &Self) -> Ordering {
            match (self.value(), other.value()) {
                (Bound::Included(a), Bound::Included(b)) | (Bound::Excluded(a), Bound::Excluded(b)) => a.cmp(&b),
                // Range patterns cannot be unbounded (yet)
                (Bound::Unbounded, _) | (_, Bound::Unbounded) => unimplemented!(),
                (Bound::Included(a), Bound::Excluded(b)) => match a.cmp(&b) {
                    Ordering::Equal => Ordering::Greater,
                    other => other,
                },
                (Bound::Excluded(a), Bound::Included(b)) => match a.cmp(&b) {
                    Ordering::Equal => Ordering::Less,
                    other => other,
                },
            }
        }
    }

    let mut values = Vec::with_capacity(2 * ranges.len());

    for r in ranges {
        values.push(Kind::Start(r.node.0, r));
        values.push(Kind::End(r.node.1, r));
    }

    values.sort();

    for (a, b) in values.iter().zip(values.iter().skip(1)) {
        match (a, b) {
            (&Kind::Start(_, ra), &Kind::End(_, rb)) => if ra.node != rb.node {
                return Some((ra, rb));
            },
            (&Kind::End(a, _), &Kind::Start(b, _)) if a != Bound::Included(b) => (),
            _ => return Some((a.range(), b.range())),
        }
    }

    None
}
