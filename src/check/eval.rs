use std::collections::btree_map::BTreeMap;

use crate::check::monad::{MetaSolution, TCS};
use crate::syntax::abs::Abs;
use crate::syntax::common::{Ident, DBI};
use crate::syntax::core::{LiftEx, Neutral, Val, ValInfo};

/// Ensure `abs` is well-typed before invoking this,
/// otherwise this function may panic or produce ill-typed core term.
fn evaluate(mut tcs: TCS, abs: Abs) -> (ValInfo, TCS) {
    use Abs::*;
    match abs {
        Type(info, level) => (Val::Type(level).into_info(info), tcs),
        Bot(info) => (Val::bot().into_info(info), tcs),
        Var(ident, _, i) => {
            let resolved = tcs.local_val(i).ast.clone().attach_dbi(i);
            (resolved.into_info(ident.info), tcs)
        }
        Ref(ident, dbi) => (tcs.glob_val(dbi).ast.clone().into_info(ident.info), tcs),
        Cons(info) => (compile_cons(info), tcs),
        App(info, f, a) => {
            // The function should always be compiled to DBI-based terms
            let (f, tcs) = evaluate(tcs, *f);
            let (a, tcs) = evaluate(tcs, *a);
            let (f, tcs) = tcs.expand_global(f.ast);
            let applied = f.apply(a.ast);
            (applied.into_info(info), tcs)
        }
        Dt(info, kind, _, param_ty, ret_ty) => {
            let (param_ty, tcs) = evaluate(tcs, *param_ty);
            let (ret_ty, tcs) = evaluate(tcs, *ret_ty);
            let term = Val::dependent_type(kind, param_ty.ast, ret_ty.ast);
            (term.into_info(info), tcs)
        }
        Pair(info, a, b) => {
            let (a, tcs) = evaluate(tcs, *a);
            let (b, tcs) = evaluate(tcs, *b);
            (Val::pair(a.ast, b.ast).into_info(info), tcs)
        }
        Fst(info, p) => {
            let (p, tcs) = evaluate(tcs, *p);
            let (p, tcs) = tcs.expand_global(p.ast);
            (p.first().into_info(info), tcs)
        }
        Snd(info, p) => {
            let (p, tcs) = evaluate(tcs, *p);
            let (p, tcs) = tcs.expand_global(p.ast);
            (p.second().into_info(info), tcs)
        }
        // This branch is not likely to be reached.
        Lam(info, _, _, body) => {
            let (body, tcs) = evaluate(tcs, *body);
            (body.ast.into_info(info), tcs)
        }
        Lift(info, levels, expr) => {
            let (expr, tcs) = evaluate(tcs, *expr);
            let (expr, tcs) = tcs.expand_global(expr.ast);
            (expr.lift(levels).into_info(info), tcs)
        }
        Meta(ident, mi) => (Val::meta(mi).into_info(ident.info), tcs),
        RowPoly(_, _, _, _) => unimplemented!(),
    }
}

/// Expand global references to concrete values,
/// like meta references or global references due to recursion.
fn expand_global(tcs: TCS, expr: Val) -> (Val, TCS) {
    let val = expr.map_neutral(&mut |neut| match neut {
        Neutral::Ref(index) => tcs.glob_val(index).ast.clone(),
        Neutral::Meta(mi) => match &tcs.meta_solutions()[mi.0] {
            MetaSolution::Solved(val) => *val.clone(),
            MetaSolution::Unsolved => panic!("Cannot eval unsolved meta: {:?}", mi),
            MetaSolution::Inlined => unreachable!(),
        },
        neut => Val::Neut(neut),
    });
    (val, tcs)
}

/// Require `Box<Val>` because currently both two calls to this function have boxed value.
pub fn expand_meta(local_vars: &[ValInfo], solution: Box<Val>) -> Val {
    local_vars
        .iter()
        // The right most local var has dbi 0.
        .fold(*solution, |ret, var| ret.apply_borrow(&var.ast))
}

pub fn compile_variant(info: Ident) -> ValInfo {
    let mut variant = BTreeMap::default();
    let mut text = info.text;
    text.remove(0);
    variant.insert(text, Val::var(DBI(0)));
    Val::lam(Val::Sum(variant)).into_info(info.info)
}

pub fn compile_cons(info: Ident) -> ValInfo {
    let mut text = info.text;
    text.remove(0);
    Val::lam(Val::cons(text, Val::var(DBI(0)))).into_info(info.info)
}

/// So you can do some functional programming based on method call chains.
impl TCS {
    /// Should be invoked **only** during type-checking,
    /// produce uid-based terms (which can be further type-checked).
    #[inline]
    pub fn evaluate(self, abs: Abs) -> (ValInfo, Self) {
        evaluate(self, abs)
    }

    #[inline]
    pub fn expand_global(self, expr: Val) -> (Val, TCS) {
        expand_global(self, expr)
    }
}
