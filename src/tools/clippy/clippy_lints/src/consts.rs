#![allow(cast_possible_truncation)]
#![allow(float_cmp)]

use rustc::lint::LateContext;
use rustc::hir::def::Def;
use rustc::hir::*;
use rustc::ty::{self, Ty, TyCtxt, Instance};
use rustc::ty::subst::{Subst, Substs};
use std::cmp::Ordering::{self, Equal};
use std::cmp::PartialOrd;
use std::hash::{Hash, Hasher};
use std::mem;
use std::rc::Rc;
use syntax::ast::{FloatTy, LitKind};
use syntax::ptr::P;
use rustc::middle::const_val::ConstVal;
use utils::{sext, unsext, clip};

#[derive(Debug, Copy, Clone)]
pub enum FloatWidth {
    F32,
    F64,
    Any,
}

impl From<FloatTy> for FloatWidth {
    fn from(ty: FloatTy) -> Self {
        match ty {
            FloatTy::F32 => FloatWidth::F32,
            FloatTy::F64 => FloatWidth::F64,
        }
    }
}

/// A `LitKind`-like enum to fold constant `Expr`s into.
#[derive(Debug, Clone)]
pub enum Constant {
    /// a String "abc"
    Str(String),
    /// a Binary String b"abc"
    Binary(Rc<Vec<u8>>),
    /// a single char 'a'
    Char(char),
    /// an integer's bit representation
    Int(u128),
    /// an f32
    F32(f32),
    /// an f64
    F64(f64),
    /// true or false
    Bool(bool),
    /// an array of constants
    Vec(Vec<Constant>),
    /// also an array, but with only one constant, repeated N times
    Repeat(Box<Constant>, u64),
    /// a tuple of constants
    Tuple(Vec<Constant>),
}

impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Constant::Str(ref ls), &Constant::Str(ref rs)) => ls == rs,
            (&Constant::Binary(ref l), &Constant::Binary(ref r)) => l == r,
            (&Constant::Char(l), &Constant::Char(r)) => l == r,
            (&Constant::Int(l), &Constant::Int(r)) => l == r,
            (&Constant::F64(l), &Constant::F64(r)) => {
                // we want `Fw32 == FwAny` and `FwAny == Fw64`, by transitivity we must have
                // `Fw32 == Fw64` so don’t compare them
                // mem::transmute is required to catch non-matching 0.0, -0.0, and NaNs
                unsafe { mem::transmute::<f64, u64>(l) == mem::transmute::<f64, u64>(r) }
            },
            (&Constant::F32(l), &Constant::F32(r)) => {
                // we want `Fw32 == FwAny` and `FwAny == Fw64`, by transitivity we must have
                // `Fw32 == Fw64` so don’t compare them
                // mem::transmute is required to catch non-matching 0.0, -0.0, and NaNs
                unsafe { mem::transmute::<f64, u64>(f64::from(l)) == mem::transmute::<f64, u64>(f64::from(r)) }
            },
            (&Constant::Bool(l), &Constant::Bool(r)) => l == r,
            (&Constant::Vec(ref l), &Constant::Vec(ref r)) | (&Constant::Tuple(ref l), &Constant::Tuple(ref r)) => l == r,
            (&Constant::Repeat(ref lv, ref ls), &Constant::Repeat(ref rv, ref rs)) => ls == rs && lv == rv,
            _ => false, // TODO: Are there inter-type equalities?
        }
    }
}

impl Hash for Constant {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match *self {
            Constant::Str(ref s) => {
                s.hash(state);
            },
            Constant::Binary(ref b) => {
                b.hash(state);
            },
            Constant::Char(c) => {
                c.hash(state);
            },
            Constant::Int(i) => {
                i.hash(state);
            },
            Constant::F32(f) => {
                unsafe { mem::transmute::<f64, u64>(f64::from(f)) }.hash(state);
            },
            Constant::F64(f) => {
                unsafe { mem::transmute::<f64, u64>(f) }.hash(state);
            },
            Constant::Bool(b) => {
                b.hash(state);
            },
            Constant::Vec(ref v) | Constant::Tuple(ref v) => {
                v.hash(state);
            },
            Constant::Repeat(ref c, l) => {
                c.hash(state);
                l.hash(state);
            },
        }
    }
}

impl PartialOrd for Constant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (&Constant::Str(ref ls), &Constant::Str(ref rs)) => Some(ls.cmp(rs)),
            (&Constant::Char(ref l), &Constant::Char(ref r)) => Some(l.cmp(r)),
            (&Constant::Int(l), &Constant::Int(r)) => Some(l.cmp(&r)),
            (&Constant::F64(l), &Constant::F64(r)) => l.partial_cmp(&r),
            (&Constant::F32(l), &Constant::F32(r)) => l.partial_cmp(&r),
            (&Constant::Bool(ref l), &Constant::Bool(ref r)) => Some(l.cmp(r)),
            (&Constant::Tuple(ref l), &Constant::Tuple(ref r)) | (&Constant::Vec(ref l), &Constant::Vec(ref r)) => {
                l.partial_cmp(r)
            },
            (&Constant::Repeat(ref lv, ref ls), &Constant::Repeat(ref rv, ref rs)) => match lv.partial_cmp(rv) {
                Some(Equal) => Some(ls.cmp(rs)),
                x => x,
            },
            _ => None, // TODO: Are there any useful inter-type orderings?
        }
    }
}

/// parse a `LitKind` to a `Constant`
pub fn lit_to_constant<'tcx>(lit: &LitKind, ty: Ty<'tcx>) -> Constant {
    use syntax::ast::*;

    match *lit {
        LitKind::Str(ref is, _) => Constant::Str(is.to_string()),
        LitKind::Byte(b) => Constant::Int(u128::from(b)),
        LitKind::ByteStr(ref s) => Constant::Binary(Rc::clone(s)),
        LitKind::Char(c) => Constant::Char(c),
        LitKind::Int(n, _) => Constant::Int(n),
        LitKind::Float(ref is, _) |
        LitKind::FloatUnsuffixed(ref is) => match ty.sty {
            ty::TyFloat(FloatTy::F32) => Constant::F32(is.as_str().parse().unwrap()),
            ty::TyFloat(FloatTy::F64) => Constant::F64(is.as_str().parse().unwrap()),
            _ => bug!(),
        },
        LitKind::Bool(b) => Constant::Bool(b),
    }
}

pub fn constant(lcx: &LateContext, e: &Expr) -> Option<(Constant, bool)> {
    let mut cx = ConstEvalLateContext {
        tcx: lcx.tcx,
        tables: lcx.tables,
        param_env: lcx.param_env,
        needed_resolution: false,
        substs: lcx.tcx.intern_substs(&[]),
    };
    cx.expr(e).map(|cst| (cst, cx.needed_resolution))
}

pub fn constant_simple(lcx: &LateContext, e: &Expr) -> Option<Constant> {
    constant(lcx, e).and_then(|(cst, res)| if res { None } else { Some(cst) })
}

/// Creates a `ConstEvalLateContext` from the given `LateContext` and `TypeckTables`
pub fn constant_context<'c, 'cc>(lcx: &LateContext<'c, 'cc>, tables: &'cc ty::TypeckTables<'cc>) -> ConstEvalLateContext<'c, 'cc> {
    ConstEvalLateContext {
        tcx: lcx.tcx,
        tables,
        param_env: lcx.param_env,
        needed_resolution: false,
        substs: lcx.tcx.intern_substs(&[]),
    }
}

pub struct ConstEvalLateContext<'a, 'tcx: 'a> {
    tcx: TyCtxt<'a, 'tcx, 'tcx>,
    tables: &'a ty::TypeckTables<'tcx>,
    param_env: ty::ParamEnv<'tcx>,
    needed_resolution: bool,
    substs: &'tcx Substs<'tcx>,
}

impl<'c, 'cc> ConstEvalLateContext<'c, 'cc> {
    /// simple constant folding: Insert an expression, get a constant or none.
    pub fn expr(&mut self, e: &Expr) -> Option<Constant> {
        match e.node {
            ExprPath(ref qpath) => self.fetch_path(qpath, e.hir_id),
            ExprBlock(ref block) => self.block(block),
            ExprIf(ref cond, ref then, ref otherwise) => self.ifthenelse(cond, then, otherwise),
            ExprLit(ref lit) => Some(lit_to_constant(&lit.node, self.tables.expr_ty(e))),
            ExprArray(ref vec) => self.multi(vec).map(Constant::Vec),
            ExprTup(ref tup) => self.multi(tup).map(Constant::Tuple),
            ExprRepeat(ref value, _) => {
                let n = match self.tables.expr_ty(e).sty {
                    ty::TyArray(_, n) => n.val.to_raw_bits().expect("array length"),
                    _ => span_bug!(e.span, "typeck error"),
                };
                self.expr(value).map(|v| Constant::Repeat(Box::new(v), n as u64))
            },
            ExprUnary(op, ref operand) => self.expr(operand).and_then(|o| match op {
                UnNot => self.constant_not(&o, self.tables.expr_ty(e)),
                UnNeg => self.constant_negate(&o, self.tables.expr_ty(e)),
                UnDeref => Some(o),
            }),
            ExprBinary(op, ref left, ref right) => self.binop(op, left, right),
            // TODO: add other expressions
            _ => None,
        }
    }

    fn constant_not(&self, o: &Constant, ty: ty::Ty) -> Option<Constant> {
        use self::Constant::*;
        match *o {
            Bool(b) => Some(Bool(!b)),
            Int(value) => {
                let mut value = !value;
                match ty.sty {
                    ty::TyInt(ity) => Some(Int(unsext(self.tcx, value as i128, ity))),
                    ty::TyUint(ity) => Some(Int(clip(self.tcx, value, ity))),
                    _ => None,
                }
            },
            _ => None,
        }
    }

    fn constant_negate(&self, o: &Constant, ty: ty::Ty) -> Option<Constant> {
        use self::Constant::*;
        match *o {
            Int(value) => {
                let ity = match ty.sty {
                    ty::TyInt(ity) => ity,
                    _ => return None,
                };
                // sign extend
                let value = sext(self.tcx, value, ity);
                let value = value.checked_neg()?;
                // clear unused bits
                Some(Int(unsext(self.tcx, value, ity)))
            },
            F32(f) => Some(F32(-f)),
            F64(f) => Some(F64(-f)),
            _ => None,
        }
    }

    /// create `Some(Vec![..])` of all constants, unless there is any
    /// non-constant part
    fn multi(&mut self, vec: &[Expr]) -> Option<Vec<Constant>> {
        vec.iter()
            .map(|elem| self.expr(elem))
            .collect::<Option<_>>()
    }

    /// lookup a possibly constant expression from a ExprPath
    fn fetch_path(&mut self, qpath: &QPath, id: HirId) -> Option<Constant> {
        let def = self.tables.qpath_def(qpath, id);
        match def {
            Def::Const(def_id) | Def::AssociatedConst(def_id) => {
                let substs = self.tables.node_substs(id);
                let substs = if self.substs.is_empty() {
                    substs
                } else {
                    substs.subst(self.tcx, self.substs)
                };
                let instance = Instance::resolve(self.tcx, self.param_env, def_id, substs)?;
                let gid = GlobalId {
                    instance,
                    promoted: None,
                };
                use rustc::mir::interpret::GlobalId;
                let result = self.tcx.const_eval(self.param_env.and(gid)).ok()?;
                let ret = miri_to_const(self.tcx, result);
                if ret.is_some() {
                    self.needed_resolution = true;
                }
                return ret;
            },
            _ => {},
        }
        None
    }

    /// A block can only yield a constant if it only has one constant expression
    fn block(&mut self, block: &Block) -> Option<Constant> {
        if block.stmts.is_empty() {
            block.expr.as_ref().and_then(|b| self.expr(b))
        } else {
            None
        }
    }

    fn ifthenelse(&mut self, cond: &Expr, then: &P<Expr>, otherwise: &Option<P<Expr>>) -> Option<Constant> {
        if let Some(Constant::Bool(b)) = self.expr(cond) {
            if b {
                self.expr(&**then)
            } else {
                otherwise.as_ref().and_then(|expr| self.expr(expr))
            }
        } else {
            None
        }
    }

    fn binop(&mut self, op: BinOp, left: &Expr, right: &Expr) -> Option<Constant> {
        let l = self.expr(left)?;
        let r = self.expr(right);
        match (l, r) {
            (Constant::Int(l), Some(Constant::Int(r))) => {
                match self.tables.expr_ty(left).sty {
                    ty::TyInt(ity) => {
                        let l = sext(self.tcx, l, ity);
                        let r = sext(self.tcx, r, ity);
                        let zext = |n: i128| Constant::Int(unsext(self.tcx, n, ity));
                        match op.node {
                            BiAdd => l.checked_add(r).map(zext),
                            BiSub => l.checked_sub(r).map(zext),
                            BiMul => l.checked_mul(r).map(zext),
                            BiDiv if r != 0 => l.checked_div(r).map(zext),
                            BiRem if r != 0 => l.checked_rem(r).map(zext),
                            BiShr => l.checked_shr(r as u128 as u32).map(zext),
                            BiShl => l.checked_shl(r as u128 as u32).map(zext),
                            BiBitXor => Some(zext(l ^ r)),
                            BiBitOr => Some(zext(l | r)),
                            BiBitAnd => Some(zext(l & r)),
                            BiEq => Some(Constant::Bool(l == r)),
                            BiNe => Some(Constant::Bool(l != r)),
                            BiLt => Some(Constant::Bool(l < r)),
                            BiLe => Some(Constant::Bool(l <= r)),
                            BiGe => Some(Constant::Bool(l >= r)),
                            BiGt => Some(Constant::Bool(l > r)),
                            _ => None,
                        }
                    }
                    ty::TyUint(_) => {
                        match op.node {
                            BiAdd => l.checked_add(r).map(Constant::Int),
                            BiSub => l.checked_sub(r).map(Constant::Int),
                            BiMul => l.checked_mul(r).map(Constant::Int),
                            BiDiv => l.checked_div(r).map(Constant::Int),
                            BiRem => l.checked_rem(r).map(Constant::Int),
                            BiShr => l.checked_shr(r as u32).map(Constant::Int),
                            BiShl => l.checked_shl(r as u32).map(Constant::Int),
                            BiBitXor => Some(Constant::Int(l ^ r)),
                            BiBitOr => Some(Constant::Int(l | r)),
                            BiBitAnd => Some(Constant::Int(l & r)),
                            BiEq => Some(Constant::Bool(l == r)),
                            BiNe => Some(Constant::Bool(l != r)),
                            BiLt => Some(Constant::Bool(l < r)),
                            BiLe => Some(Constant::Bool(l <= r)),
                            BiGe => Some(Constant::Bool(l >= r)),
                            BiGt => Some(Constant::Bool(l > r)),
                            _ => None,
                        }
                    },
                    _ => None,
                }
            },
            (Constant::F32(l), Some(Constant::F32(r))) => match op.node {
                BiAdd => Some(Constant::F32(l + r)),
                BiSub => Some(Constant::F32(l - r)),
                BiMul => Some(Constant::F32(l * r)),
                BiDiv => Some(Constant::F32(l / r)),
                BiRem => Some(Constant::F32(l % r)),
                BiEq => Some(Constant::Bool(l == r)),
                BiNe => Some(Constant::Bool(l != r)),
                BiLt => Some(Constant::Bool(l < r)),
                BiLe => Some(Constant::Bool(l <= r)),
                BiGe => Some(Constant::Bool(l >= r)),
                BiGt => Some(Constant::Bool(l > r)),
                _ => None,
            },
            (Constant::F64(l), Some(Constant::F64(r))) => match op.node {
                BiAdd => Some(Constant::F64(l + r)),
                BiSub => Some(Constant::F64(l - r)),
                BiMul => Some(Constant::F64(l * r)),
                BiDiv => Some(Constant::F64(l / r)),
                BiRem => Some(Constant::F64(l % r)),
                BiEq => Some(Constant::Bool(l == r)),
                BiNe => Some(Constant::Bool(l != r)),
                BiLt => Some(Constant::Bool(l < r)),
                BiLe => Some(Constant::Bool(l <= r)),
                BiGe => Some(Constant::Bool(l >= r)),
                BiGt => Some(Constant::Bool(l > r)),
                _ => None,
            },
            (l, r) => match (op.node, l, r) {
                (BiAnd, Constant::Bool(false), _) => Some(Constant::Bool(false)),
                (BiOr, Constant::Bool(true), _) => Some(Constant::Bool(true)),
                (BiAnd, Constant::Bool(true), Some(r)) | (BiOr, Constant::Bool(false), Some(r)) => Some(r),
                (BiBitXor, Constant::Bool(l), Some(Constant::Bool(r))) => Some(Constant::Bool(l ^ r)),
                (BiBitAnd, Constant::Bool(l), Some(Constant::Bool(r))) => Some(Constant::Bool(l & r)),
                (BiBitOr, Constant::Bool(l), Some(Constant::Bool(r))) => Some(Constant::Bool(l | r)),
                _ => None,
            },
        }
    }
}

pub fn miri_to_const<'a, 'tcx>(tcx: TyCtxt<'a, 'tcx, 'tcx>, result: &ty::Const<'tcx>) -> Option<Constant> {
    use rustc::mir::interpret::{Value, PrimVal};
    match result.val {
        ConstVal::Value(Value::ByVal(PrimVal::Bytes(b))) => match result.ty.sty {
            ty::TyBool => Some(Constant::Bool(b == 1)),
            ty::TyUint(_) | ty::TyInt(_) => Some(Constant::Int(b)),
            ty::TyFloat(FloatTy::F32) => Some(Constant::F32(f32::from_bits(b as u32))),
            ty::TyFloat(FloatTy::F64) => Some(Constant::F64(f64::from_bits(b as u64))),
            // FIXME: implement other conversion
            _ => None,
        },
        ConstVal::Value(Value::ByValPair(PrimVal::Ptr(ptr), PrimVal::Bytes(n))) => match result.ty.sty {
            ty::TyRef(_, tam) => match tam.ty.sty {
                ty::TyStr => {
                    let alloc = tcx
                        .interpret_interner
                        .get_alloc(ptr.alloc_id)
                        .unwrap();
                    let offset = ptr.offset as usize;
                    let n = n as usize;
                    String::from_utf8(alloc.bytes[offset..(offset + n)].to_owned()).ok().map(Constant::Str)
                },
                _ => None,
            },
            _ => None,
        }
        // FIXME: implement other conversions
        _ => None,
    }
}
