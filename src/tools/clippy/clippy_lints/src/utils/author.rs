//! A group of attributes that can be attached to Rust code in order
//! to generate a clippy lint detecting said code automatically.

#![allow(print_stdout, use_debug)]

use rustc::lint::*;
use rustc::hir;
use rustc::hir::{Expr, Expr_, QPath, Ty_, Pat, PatKind, BindingAnnotation, StmtSemi, StmtExpr, StmtDecl, Decl_, Stmt};
use rustc::hir::intravisit::{NestedVisitorMap, Visitor};
use syntax::ast::{self, Attribute, LitKind, DUMMY_NODE_ID};
use std::collections::HashMap;

/// **What it does:** Generates clippy code that detects the offending pattern
///
/// **Example:**
/// ```rust
/// // ./tests/ui/my_lint.rs
/// fn foo() {
///     // detect the following pattern
///     #[clippy(author)]
///     if x == 42 {
///         // but ignore everything from here on
///         #![clippy(author = "ignore")]
///     }
/// }
/// ```
///
/// Running `TESTNAME=ui/my_lint cargo test --test compile-test` will produce
/// a `./tests/ui/new_lint.stdout` file with the generated code:
///
/// ```rust
/// // ./tests/ui/new_lint.stdout
/// if_chain!{
///     if let Expr_::ExprIf(ref cond, ref then, None) = item.node,
///     if let Expr_::ExprBinary(BinOp::Eq, ref left, ref right) = cond.node,
///     if let Expr_::ExprPath(ref path) = left.node,
///     if let Expr_::ExprLit(ref lit) = right.node,
///     if let LitKind::Int(42, _) = lit.node,
///     then {
///         // report your lint here
///     }
/// }
/// ```
declare_clippy_lint! {
    pub LINT_AUTHOR,
    internal_warn,
    "helper for writing lints"
}

pub struct Pass;

impl LintPass for Pass {
    fn get_lints(&self) -> LintArray {
        lint_array!(LINT_AUTHOR)
    }
}

fn prelude() {
    println!("if_chain! {{");
}

fn done() {
    println!("    then {{");
    println!("        // report your lint here");
    println!("    }}");
    println!("}}");
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for Pass {
    fn check_item(&mut self, _cx: &LateContext<'a, 'tcx>, item: &'tcx hir::Item) {
        if !has_attr(&item.attrs) {
            return;
        }
        prelude();
        PrintVisitor::new("item").visit_item(item);
        done();
    }

    fn check_impl_item(&mut self, _cx: &LateContext<'a, 'tcx>, item: &'tcx hir::ImplItem) {
        if !has_attr(&item.attrs) {
            return;
        }
        prelude();
        PrintVisitor::new("item").visit_impl_item(item);
        done();
    }

    fn check_trait_item(&mut self, _cx: &LateContext<'a, 'tcx>, item: &'tcx hir::TraitItem) {
        if !has_attr(&item.attrs) {
            return;
        }
        prelude();
        PrintVisitor::new("item").visit_trait_item(item);
        done();
    }

    fn check_variant(&mut self, _cx: &LateContext<'a, 'tcx>, var: &'tcx hir::Variant, generics: &hir::Generics) {
        if !has_attr(&var.node.attrs) {
            return;
        }
        prelude();
        PrintVisitor::new("var").visit_variant(var, generics, DUMMY_NODE_ID);
        done();
    }

    fn check_struct_field(&mut self, _cx: &LateContext<'a, 'tcx>, field: &'tcx hir::StructField) {
        if !has_attr(&field.attrs) {
            return;
        }
        prelude();
        PrintVisitor::new("field").visit_struct_field(field);
        done();
    }

    fn check_expr(&mut self, _cx: &LateContext<'a, 'tcx>, expr: &'tcx hir::Expr) {
        if !has_attr(&expr.attrs) {
            return;
        }
        prelude();
        PrintVisitor::new("expr").visit_expr(expr);
        done();
    }

    fn check_arm(&mut self, _cx: &LateContext<'a, 'tcx>, arm: &'tcx hir::Arm) {
        if !has_attr(&arm.attrs) {
            return;
        }
        prelude();
        PrintVisitor::new("arm").visit_arm(arm);
        done();
    }

    fn check_stmt(&mut self, _cx: &LateContext<'a, 'tcx>, stmt: &'tcx hir::Stmt) {
        if !has_attr(stmt.node.attrs()) {
            return;
        }
        prelude();
        PrintVisitor::new("stmt").visit_stmt(stmt);
        done();
    }

    fn check_foreign_item(&mut self, _cx: &LateContext<'a, 'tcx>, item: &'tcx hir::ForeignItem) {
        if !has_attr(&item.attrs) {
            return;
        }
        prelude();
        PrintVisitor::new("item").visit_foreign_item(item);
        done();
    }
}

impl PrintVisitor {
    fn new(s: &'static str) -> Self {
        Self {
            ids: HashMap::new(),
            current: s.to_owned(),
        }
    }

    fn next(&mut self, s: &'static str) -> String {
        use std::collections::hash_map::Entry::*;
        match self.ids.entry(s) {
            // already there: start numbering from `1`
            Occupied(mut occ) => {
                let val = occ.get_mut();
                *val += 1;
                format!("{}{}", s, *val)
            },
            // not there: insert and return name as given
            Vacant(vac) => {
                vac.insert(0);
                s.to_owned()
            },
        }
    }

    fn print_qpath(&mut self, path: &QPath) {
        print!("    if match_qpath({}, &[", self.current);
        print_path(path, &mut true);
        println!("]);");
    }
}

struct PrintVisitor {
    /// Fields are the current index that needs to be appended to pattern
    /// binding names
    ids: HashMap<&'static str, usize>,
    /// the name that needs to be destructured
    current: String,
}

impl<'tcx> Visitor<'tcx> for PrintVisitor {
    fn visit_expr(&mut self, expr: &Expr) {
        print!("    if let Expr_::Expr");
        let current = format!("{}.node", self.current);
        match expr.node {
            Expr_::ExprBox(ref inner) => {
                let inner_pat = self.next("inner");
                println!("Box(ref {}) = {};", inner_pat, current);
                self.current = inner_pat;
                self.visit_expr(inner);
            },
            Expr_::ExprArray(ref elements) => {
                let elements_pat = self.next("elements");
                println!("Array(ref {}) = {};", elements_pat, current);
                println!("    if {}.len() == {};", elements_pat, elements.len());
                for (i, element) in elements.iter().enumerate() {
                    self.current = format!("{}[{}]", elements_pat, i);
                    self.visit_expr(element);
                }
            },
            Expr_::ExprCall(ref _func, ref _args) => {
                println!("Call(ref func, ref args) = {};", current);
                println!("    // unimplemented: `ExprCall` is not further destructured at the moment");
            },
            Expr_::ExprMethodCall(ref _method_name, ref _generics, ref _args) => {
                println!("MethodCall(ref method_name, ref generics, ref args) = {};", current);
                println!("    // unimplemented: `ExprMethodCall` is not further destructured at the moment");
            },
            Expr_::ExprTup(ref elements) => {
                let elements_pat = self.next("elements");
                println!("Tup(ref {}) = {};", elements_pat, current);
                println!("    if {}.len() == {};", elements_pat, elements.len());
                for (i, element) in elements.iter().enumerate() {
                    self.current = format!("{}[{}]", elements_pat, i);
                    self.visit_expr(element);
                }
            },
            Expr_::ExprBinary(ref op, ref left, ref right) => {
                let op_pat = self.next("op");
                let left_pat = self.next("left");
                let right_pat = self.next("right");
                println!("Binary(ref {}, ref {}, ref {}) = {};", op_pat, left_pat, right_pat, current);
                println!("    if BinOp_::{:?} == {}.node;", op.node, op_pat);
                self.current = left_pat;
                self.visit_expr(left);
                self.current = right_pat;
                self.visit_expr(right);
            },
            Expr_::ExprUnary(ref op, ref inner) => {
                let inner_pat = self.next("inner");
                println!("Unary(UnOp::{:?}, ref {}) = {};", op, inner_pat, current);
                self.current = inner_pat;
                self.visit_expr(inner);
            },
            Expr_::ExprLit(ref lit) => {
                let lit_pat = self.next("lit");
                println!("Lit(ref {}) = {};", lit_pat, current);
                match lit.node {
                    LitKind::Bool(val) => println!("    if let LitKind::Bool({:?}) = {}.node;", val, lit_pat),
                    LitKind::Char(c) => println!("    if let LitKind::Char({:?}) = {}.node;", c, lit_pat),
                    LitKind::Byte(b) => println!("    if let LitKind::Byte({}) = {}.node;", b, lit_pat),
                    // FIXME: also check int type
                    LitKind::Int(i, _) => println!("    if let LitKind::Int({}, _) = {}.node;", i, lit_pat),
                    LitKind::Float(..) => println!("    if let LitKind::Float(..) = {}.node;", lit_pat),
                    LitKind::FloatUnsuffixed(_) => {
                        println!("    if let LitKind::FloatUnsuffixed(_) = {}.node;", lit_pat)
                    },
                    LitKind::ByteStr(ref vec) => {
                        let vec_pat = self.next("vec");
                        println!("    if let LitKind::ByteStr(ref {}) = {}.node;", vec_pat, lit_pat);
                        println!("    if let [{:?}] = **{};", vec, vec_pat);
                    },
                    LitKind::Str(ref text, _) => {
                        let str_pat = self.next("s");
                        println!("    if let LitKind::Str(ref {}) = {}.node;", str_pat, lit_pat);
                        println!("    if {}.as_str() == {:?}", str_pat, &*text.as_str())
                    },
                }
            },
            Expr_::ExprCast(ref expr, ref ty) => {
                let cast_pat = self.next("expr");
                let cast_ty = self.next("cast_ty");
                let qp_label = self.next("qp");

                println!("Cast(ref {}, ref {}) = {};", cast_pat, cast_ty, current);
                if let Ty_::TyPath(ref qp) = ty.node {
                    println!("    if let Ty_::TyPath(ref {}) = {}.node;", qp_label, cast_ty);
                    self.current = qp_label;
                    self.print_qpath(qp);
                }
                self.current = cast_pat;
                self.visit_expr(expr);
            },
            Expr_::ExprType(ref expr, ref _ty) => {
                let cast_pat = self.next("expr");
                println!("Type(ref {}, _) = {};", cast_pat, current);
                self.current = cast_pat;
                self.visit_expr(expr);
            },
            Expr_::ExprIf(ref cond, ref then, ref opt_else) => {
                let cond_pat = self.next("cond");
                let then_pat = self.next("then");
                if let Some(ref else_) = *opt_else {
                    let else_pat = self.next("else_");
                    println!("If(ref {}, ref {}, Some(ref {})) = {};", cond_pat, then_pat, else_pat, current);
                    self.current = else_pat;
                    self.visit_expr(else_);
                } else {
                    println!("If(ref {}, ref {}, None) = {};", cond_pat, then_pat, current);
                }
                self.current = cond_pat;
                self.visit_expr(cond);
                self.current = then_pat;
                self.visit_expr(then);
            },
            Expr_::ExprWhile(ref cond, ref body, _) => {
                let cond_pat = self.next("cond");
                let body_pat = self.next("body");
                let label_pat = self.next("label");
                println!("While(ref {}, ref {}, ref {}) = {};", cond_pat, body_pat, label_pat, current);
                self.current = cond_pat;
                self.visit_expr(cond);
                self.current = body_pat;
                self.visit_block(body);
            },
            Expr_::ExprLoop(ref body, _, desugaring) => {
                let body_pat = self.next("body");
                let des = loop_desugaring_name(desugaring);
                let label_pat = self.next("label");
                println!("Loop(ref {}, ref {}, {}) = {};", body_pat, label_pat, des, current);
                self.current = body_pat;
                self.visit_block(body);
            },
            Expr_::ExprMatch(ref expr, ref arms, desugaring) => {
                let des = desugaring_name(desugaring);
                let expr_pat = self.next("expr");
                let arms_pat = self.next("arms");
                println!("Match(ref {}, ref {}, {}) = {};", expr_pat, arms_pat, des, current);
                self.current = expr_pat;
                self.visit_expr(expr);
                println!("    if {}.len() == {};", arms_pat, arms.len());
                for (i, arm) in arms.iter().enumerate() {
                    self.current = format!("{}[{}].body", arms_pat, i);
                    self.visit_expr(&arm.body);
                    if let Some(ref guard) = arm.guard {
                        let guard_pat = self.next("guard");
                        println!("    if let Some(ref {}) = {}[{}].guard", guard_pat, arms_pat, i);
                        self.current = guard_pat;
                        self.visit_expr(guard);
                    }
                    println!("    if {}[{}].pats.len() == {};", arms_pat, i, arm.pats.len());
                    for (j, pat) in arm.pats.iter().enumerate() {
                        self.current = format!("{}[{}].pats[{}]", arms_pat, i, j);
                        self.visit_pat(pat);
                    }
                }
            },
            Expr_::ExprClosure(ref _capture_clause, ref _func, _, _, _) => {
                println!("Closure(ref capture_clause, ref func, _, _, _) = {};", current);
                println!("    // unimplemented: `ExprClosure` is not further destructured at the moment");
            },
            Expr_::ExprYield(ref sub) => {
                let sub_pat = self.next("sub");
                println!("Yield(ref sub) = {};", current);
                self.current = sub_pat;
                self.visit_expr(sub);
            },
            Expr_::ExprBlock(ref block, _) => {
                let block_pat = self.next("block");
                println!("Block(ref {}) = {};", block_pat, current);
                self.current = block_pat;
                self.visit_block(block);
            },
            Expr_::ExprAssign(ref target, ref value) => {
                let target_pat = self.next("target");
                let value_pat = self.next("value");
                println!("Assign(ref {}, ref {}) = {};", target_pat, value_pat, current);
                self.current = target_pat;
                self.visit_expr(target);
                self.current = value_pat;
                self.visit_expr(value);
            },
            Expr_::ExprAssignOp(ref op, ref target, ref value) => {
                let op_pat = self.next("op");
                let target_pat = self.next("target");
                let value_pat = self.next("value");
                println!("AssignOp(ref {}, ref {}, ref {}) = {};", op_pat, target_pat, value_pat, current);
                println!("    if BinOp_::{:?} == {}.node;", op.node, op_pat);
                self.current = target_pat;
                self.visit_expr(target);
                self.current = value_pat;
                self.visit_expr(value);
            },
            Expr_::ExprField(ref object, ref field_name) => {
                let obj_pat = self.next("object");
                let field_name_pat = self.next("field_name");
                println!("Field(ref {}, ref {}) = {};", obj_pat, field_name_pat, current);
                println!("    if {}.node.as_str() == {:?}", field_name_pat, field_name.node.as_str());
                self.current = obj_pat;
                self.visit_expr(object);
            },
            Expr_::ExprIndex(ref object, ref index) => {
                let object_pat = self.next("object");
                let index_pat = self.next("index");
                println!("Index(ref {}, ref {}) = {};", object_pat, index_pat, current);
                self.current = object_pat;
                self.visit_expr(object);
                self.current = index_pat;
                self.visit_expr(index);
            },
            Expr_::ExprPath(ref path) => {
                let path_pat = self.next("path");
                println!("Path(ref {}) = {};", path_pat, current);
                self.current = path_pat;
                self.print_qpath(path);
            },
            Expr_::ExprAddrOf(mutability, ref inner) => {
                let inner_pat = self.next("inner");
                println!("AddrOf({:?}, ref {}) = {};", mutability, inner_pat, current);
                self.current = inner_pat;
                self.visit_expr(inner);
            },
            Expr_::ExprBreak(ref _destination, ref opt_value) => {
                let destination_pat = self.next("destination");
                if let Some(ref value) = *opt_value {
                    let value_pat = self.next("value");
                    println!("Break(ref {}, Some(ref {})) = {};", destination_pat, value_pat, current);
                    self.current = value_pat;
                    self.visit_expr(value);
                } else {
                    println!("Break(ref {}, None) = {};", destination_pat, current);
                }
                // FIXME: implement label printing
            },
            Expr_::ExprAgain(ref _destination) => {
                let destination_pat = self.next("destination");
                println!("Again(ref {}) = {};", destination_pat, current);
                // FIXME: implement label printing
            },
            Expr_::ExprRet(ref opt_value) => if let Some(ref value) = *opt_value {
                let value_pat = self.next("value");
                println!("Ret(Some(ref {})) = {};", value_pat, current);
                self.current = value_pat;
                self.visit_expr(value);
            } else {
                println!("Ret(None) = {};", current);
            },
            Expr_::ExprInlineAsm(_, ref _input, ref _output) => {
                println!("InlineAsm(_, ref input, ref output) = {};", current);
                println!("    // unimplemented: `ExprInlineAsm` is not further destructured at the moment");
            },
            Expr_::ExprStruct(ref path, ref fields, ref opt_base) => {
                let path_pat = self.next("path");
                let fields_pat = self.next("fields");
                if let Some(ref base) = *opt_base {
                    let base_pat = self.next("base");
                    println!(
                        "Struct(ref {}, ref {}, Some(ref {})) = {};",
                        path_pat,
                        fields_pat,
                        base_pat,
                        current
                    );
                    self.current = base_pat;
                    self.visit_expr(base);
                } else {
                    println!("Struct(ref {}, ref {}, None) = {};", path_pat, fields_pat, current);
                }
                self.current = path_pat;
                self.print_qpath(path);
                println!("    if {}.len() == {};", fields_pat, fields.len());
                println!("    // unimplemented: field checks");
            },
            // FIXME: compute length (needs type info)
            Expr_::ExprRepeat(ref value, _) => {
                let value_pat = self.next("value");
                println!("Repeat(ref {}, _) = {};", value_pat, current);
                println!("// unimplemented: repeat count check");
                self.current = value_pat;
                self.visit_expr(value);
            },
        }
    }

    fn visit_pat(&mut self, pat: &Pat) {
        print!("    if let PatKind::");
        let current = format!("{}.node", self.current);
        match pat.node {
            PatKind::Wild => println!("Wild = {};", current),
            PatKind::Binding(anno, _, name, ref sub) => {
                let anno_pat = match anno {
                    BindingAnnotation::Unannotated => "BindingAnnotation::Unannotated",
                    BindingAnnotation::Mutable => "BindingAnnotation::Mutable",
                    BindingAnnotation::Ref => "BindingAnnotation::Ref",
                    BindingAnnotation::RefMut => "BindingAnnotation::RefMut",
                };
                let name_pat = self.next("name");
                if let Some(ref sub) = *sub {
                    let sub_pat = self.next("sub");
                    println!("Binding({}, _, {}, Some(ref {})) = {};", anno_pat, name_pat, sub_pat, current);
                    self.current = sub_pat;
                    self.visit_pat(sub);
                } else {
                    println!("Binding({}, _, {}, None) = {};", anno_pat, name_pat, current);
                }
                println!("    if {}.node.as_str() == \"{}\";", name_pat, name.node.as_str());
            }
            PatKind::Struct(ref path, ref fields, ignore) => {
                let path_pat = self.next("path");
                let fields_pat = self.next("fields");
                println!("Struct(ref {}, ref {}, {}) = {};", path_pat, fields_pat, ignore, current);
                self.current = path_pat;
                self.print_qpath(path);
                println!("    if {}.len() == {};", fields_pat, fields.len());
                println!("    // unimplemented: field checks");
            }
            PatKind::TupleStruct(ref path, ref fields, skip_pos) => {
                let path_pat = self.next("path");
                let fields_pat = self.next("fields");
                println!("TupleStruct(ref {}, ref {}, {:?}) = {};", path_pat, fields_pat, skip_pos, current);
                self.current = path_pat;
                self.print_qpath(path);
                println!("    if {}.len() == {};", fields_pat, fields.len());
                println!("    // unimplemented: field checks");
            },
            PatKind::Path(ref path) => {
                let path_pat = self.next("path");
                println!("Path(ref {}) = {};", path_pat, current);
                self.current = path_pat;
                self.print_qpath(path);
            }
            PatKind::Tuple(ref fields, skip_pos) => {
                let fields_pat = self.next("fields");
                println!("Tuple(ref {}, {:?}) = {};", fields_pat, skip_pos, current);
                println!("    if {}.len() == {};", fields_pat, fields.len());
                println!("    // unimplemented: field checks");
            }
            PatKind::Box(ref pat) => {
                let pat_pat = self.next("pat");
                println!("Box(ref {}) = {};", pat_pat, current);
                self.current = pat_pat;
                self.visit_pat(pat);
            },
            PatKind::Ref(ref pat, muta) => {
                let pat_pat = self.next("pat");
                println!("Ref(ref {}, Mutability::{:?}) = {};", pat_pat, muta, current);
                self.current = pat_pat;
                self.visit_pat(pat);
            },
            PatKind::Lit(ref lit_expr) => {
                let lit_expr_pat = self.next("lit_expr");
                println!("Lit(ref {}) = {}", lit_expr_pat, current);
                self.current = lit_expr_pat;
                self.visit_expr(lit_expr);
            }
            PatKind::Range(ref start, ref end, end_kind) => {
                let start_pat = self.next("start");
                let end_pat = self.next("end");
                println!("Range(ref {}, ref {}, RangeEnd::{:?}) = {};", start_pat, end_pat, end_kind, current);
                self.current = start_pat;
                self.visit_expr(start);
                self.current = end_pat;
                self.visit_expr(end);
            }
            PatKind::Slice(ref start, ref middle, ref end) => {
                let start_pat = self.next("start");
                let end_pat = self.next("end");
                if let Some(ref middle) = middle {
                    let middle_pat = self.next("middle");
                    println!("Slice(ref {}, Some(ref {}), ref {}) = {};", start_pat, middle_pat, end_pat, current);
                    self.current = middle_pat;
                    self.visit_pat(middle);
                } else {
                    println!("Slice(ref {}, None, ref {}) = {};", start_pat, end_pat, current);
                }
                println!("    if {}.len() == {};", start_pat, start.len());
                for (i, pat) in start.iter().enumerate() {
                    self.current = format!("{}[{}]", start_pat, i);
                    self.visit_pat(pat);
                }
                println!("    if {}.len() == {};", end_pat, end.len());
                for (i, pat) in end.iter().enumerate() {
                    self.current = format!("{}[{}]", end_pat, i);
                    self.visit_pat(pat);
                }
            }
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) {
        print!("    if let Stmt_::");
        let current = format!("{}.node", self.current);
        match s.node {
            // Could be an item or a local (let) binding:
            StmtDecl(ref decl, _) => {
                let decl_pat = self.next("decl");
                println!("StmtDecl(ref {}, _) = {}", decl_pat, current);
                print!("    if let Decl_::");
                let current = format!("{}.node", decl_pat);
                match decl.node {
                    // A local (let) binding:
                    Decl_::DeclLocal(ref local) => {
                        let local_pat = self.next("local");
                        println!("DeclLocal(ref {}) = {};", local_pat, current);
                        if let Some(ref init) = local.init {
                            let init_pat = self.next("init");
                            println!("    if let Some(ref {}) = {}.init", init_pat, local_pat);
                            self.current = init_pat;
                            self.visit_expr(init);
                        }
                        self.current = format!("{}.pat", local_pat);
                        self.visit_pat(&local.pat);
                    },
                    // An item binding:
                    Decl_::DeclItem(_) => {
                        println!("DeclItem(item_id) = {};", current);
                    },
                }
            }

            // Expr without trailing semi-colon (must have unit type):
            StmtExpr(ref e, _) => {
                let e_pat = self.next("e");
                println!("StmtExpr(ref {}, _) = {}", e_pat, current);
                self.current = e_pat;
                self.visit_expr(e);
            },

            // Expr with trailing semi-colon (may have any type):
            StmtSemi(ref e, _) => {
                let e_pat = self.next("e");
                println!("StmtSemi(ref {}, _) = {}", e_pat, current);
                self.current = e_pat;
                self.visit_expr(e);
            },
        }
    }

    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, 'tcx> {
        NestedVisitorMap::None
    }
}

fn has_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.check_name("clippy") && attr.meta_item_list().map_or(false, |list| {
            list.len() == 1 && match list[0].node {
                ast::NestedMetaItemKind::MetaItem(ref it) => it.name() == "author",
                ast::NestedMetaItemKind::Literal(_) => false,
            }
        })
    })
}

fn desugaring_name(des: hir::MatchSource) -> String {
    match des {
        hir::MatchSource::ForLoopDesugar => "MatchSource::ForLoopDesugar".to_string(),
        hir::MatchSource::TryDesugar => "MatchSource::TryDesugar".to_string(),
        hir::MatchSource::WhileLetDesugar => "MatchSource::WhileLetDesugar".to_string(),
        hir::MatchSource::Normal => "MatchSource::Normal".to_string(),
        hir::MatchSource::IfLetDesugar { contains_else_clause } => format!("MatchSource::IfLetDesugar {{ contains_else_clause: {} }}", contains_else_clause),
    }
}

fn loop_desugaring_name(des: hir::LoopSource) -> &'static str {
    match des {
        hir::LoopSource::ForLoop => "LoopSource::ForLoop",
        hir::LoopSource::Loop => "LoopSource::Loop",
        hir::LoopSource::WhileLet => "LoopSource::WhileLet",
    }
}

fn print_path(path: &QPath, first: &mut bool) {
    match *path {
        QPath::Resolved(_, ref path) => for segment in &path.segments {
            if *first {
                *first = false;
            } else {
                print!(", ");
            }
            print!("{:?}", segment.name.as_str());
        },
        QPath::TypeRelative(ref ty, ref segment) => match ty.node {
            hir::Ty_::TyPath(ref inner_path) => {
                print_path(inner_path, first);
                if *first {
                    *first = false;
                } else {
                    print!(", ");
                }
                print!("{:?}", segment.name.as_str());
            },
            ref other => print!("/* unimplemented: {:?}*/", other),
        },
    }
}
