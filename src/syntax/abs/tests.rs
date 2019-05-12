use super::{trans_decls, AbsDecl};
use crate::syntax::abs::{trans_expr, Abs};
use crate::syntax::common::{DtKind, ToSyntaxInfo};
use crate::syntax::surf::parse_str_err_printed;

#[test]
fn trans_bot() {
    let surf = parse_str_err_printed(
        "val a : Type;\n\
         let a = !;",
    )
    .unwrap();
    let mut ctx = trans_decls(surf).unwrap();
    assert_eq!(1, ctx.len());
    let decl = ctx.pop().unwrap();
    println!("{:?}", decl);
    match decl {
        AbsDecl::Both(_ty_abs, _abs) => {}
        _ => panic!(),
    };
}

#[test]
fn many_decls() {
    let surf = parse_str_err_printed(
        "val a : Type1;\n\
         let a = Type;\n\
         val b : Type1;\n\
         let b = Type;",
    )
    .unwrap();
    let mut ctx = trans_decls(surf).unwrap();
    assert_eq!(2, ctx.len());
    let decl = ctx.pop().unwrap();
    println!("{:?}", decl);
    match decl {
        AbsDecl::Both(_ty_abs, _abs) => {}
        _ => panic!(),
    };
    let decl = ctx.pop().unwrap();
    println!("{:?}", decl);
    match decl {
        AbsDecl::Both(_ty_abs, _abs) => {}
        _ => panic!(),
    };
    assert!(ctx.is_empty());
}

fn must_be_app(abs: Abs) -> Abs {
    match abs {
        Abs::App(_, _, abs) => *abs,
        e => panic!("`{:?}` is not an `Abs::App`.", e),
    }
}

fn must_be_pi(abs: Abs) -> (Abs, Abs) {
    match abs {
        Abs::Dt(_, DtKind::Pi, param, abs) => (*param, *abs),
        e => panic!("`{:?}` is not an `Abs::Dt(_, Pi, _, _)`.", e),
    }
}

fn must_be_lam(abs: Abs) -> Abs {
    match abs {
        Abs::Lam(_, _, abs) => *abs,
        e => panic!("`{:?}` is not an `Abs::Lam(_, _, _)`.", e),
    }
}

#[test]
fn trans_pi_env() {
    let pi_expr = parse_str_err_printed("val t : ((a : Type) -> (b : Type(a)) -> Type(b));")
        .unwrap()
        .remove(0)
        .body;
    let pi_expr = trans_expr(&pi_expr, &[], &Default::default()).expect("Parse failed.");
    let (_, bc) = must_be_pi(pi_expr);
    let (b, c) = must_be_pi(bc);
    // the type of `b`, `c` should be _(0), _(0)
    let b = must_be_app(b);
    let c = must_be_app(c);
    assert_eq!(Abs::Local(b.to_info(), 0), b);
    assert_eq!(Abs::Local(c.to_info(), 0), c);
}

#[test]
fn trans_pi_shadowing() {
    let code = "val t : ((a : Type) -> (b : Type(a)) -> (b: Type(b)) -> Type(a));";
    let pi_expr = parse_str_err_printed(code).unwrap().remove(0).body;
    let pi_abs = trans_expr(&pi_expr, &[], &Default::default()).unwrap();
    let (_, bc) = must_be_pi(pi_abs);
    let (b1, bc) = must_be_pi(bc);
    let (b2, c) = must_be_pi(bc);
    // the type of `b1`, `b2`, `c` should be _(0), _(0), _(2)
    let b1 = must_be_app(b1);
    let b2 = must_be_app(b2);
    let c = must_be_app(c);
    assert_eq!(Abs::Local(b1.to_info(), 0), b1);
    assert_eq!(Abs::Local(b2.to_info(), 0), b2);
    assert_eq!(Abs::Local(c.to_info(), 2), c);
}

#[test]
fn trans_lam() {
    let code = r"let l = \a . \b . \a . b a;";
    let lam_expr = parse_str_err_printed(code).unwrap().remove(0).body;
    let lam_abs = trans_expr(&lam_expr, &[], &Default::default()).unwrap();
    let abs_lam_ba = must_be_lam(lam_abs);
    let abs_lam_a = must_be_lam(abs_lam_ba);
    match must_be_lam(abs_lam_a) {
        Abs::App(_, b, a) => {
            // lam body should be App(_info, Local(_info, 1), Local(_info, 0))
            assert_eq!(Abs::Local(b.to_info(), 1), *b);
            assert_eq!(Abs::Local(a.to_info(), 0), *a);
        }
        _ => panic!(),
    }
}
