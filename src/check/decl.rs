use crate::syntax::abs::AbsDecl;

use super::monad::{TCM, TCS};
use crate::syntax::common::ToSyntaxInfo;
use crate::syntax::core::Term;

pub fn check_decls(mut tcs: TCS, decls: Vec<AbsDecl>) -> TCM {
    for decl in decls.into_iter() {
        tcs = tcs.check_decl(decl)?;
    }
    Ok(tcs)
}

fn check_decl(tcs: TCS, decl: AbsDecl) -> TCM {
    match decl {
        AbsDecl::Both(sign_abs, impl_abs) => {
            debug_assert_eq!(tcs.gamma.len(), tcs.env.len());
            let (sign_fake, tcs) = tcs.check_type(&sign_abs)?;
            let (sign, tcs) = tcs.unsafe_compile(sign_abs, Some(sign_fake.ast));
            println!("sign: {}", sign.ast);
            let (val_fake, tcs) = tcs.check(&impl_abs, &sign.ast)?;
            let (val, mut tcs) = tcs.unsafe_compile(impl_abs, Some(val_fake.ast));
            println!("body: {}", val.ast);

            tcs.gamma.push(sign);
            tcs.env.push(val);
            // Err(TCE::DbiOverflow(tcs.env.len(), new_dbi))
            Ok(tcs)
        }
        AbsDecl::Sign(sign_abs) => {
            debug_assert_eq!(tcs.gamma.len(), tcs.env.len());
            let syntax_info = sign_abs.to_info();
            let (sign_fake, tcs) = tcs.check_type(&sign_abs)?;
            let (val, mut tcs) = tcs.unsafe_compile(sign_abs, Some(sign_fake.ast));
            tcs.env.push(Term::axiom().into_info(syntax_info));
            tcs.gamma.push(val);
            // Give warning on axiom?
            Ok(tcs)
        }
        AbsDecl::Impl(impl_abs) => {
            debug_assert_eq!(tcs.gamma.len(), tcs.env.len());
            let (inferred, tcs) = tcs.infer(&impl_abs)?;
            let (compiled, mut tcs) = tcs.unsafe_compile(impl_abs, None);
            tcs.env.push(compiled);
            tcs.gamma.push(inferred);
            Ok(tcs)
        }
    }
}

impl TCS {
    #[inline]
    pub fn check_decls(self, decls: Vec<AbsDecl>) -> TCM {
        check_decls(self, decls)
    }

    #[inline]
    pub fn check_decl(self, decl: AbsDecl) -> TCM {
        check_decl(self, decl)
    }
}
