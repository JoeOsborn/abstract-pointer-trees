use std::cell::OnceCell;
use std::rc::Rc;

// Ptr and Lam must be opaque
#[derive(PartialEq, Eq, Debug)]
pub struct Ptr(Rc<OnceCell<Box<Expr>>>);
#[derive(PartialEq, Eq, Debug)]
pub struct Lam(Ptr, Box<Expr>);

// Exprs will only ever be evaluated once,
// so Box is used instead of Rc
#[derive(PartialEq, Eq, Debug)]
pub enum Expr {
    Ptr(Ptr),
    Bas(&'static str),
    Lam(Lam),
    App(Box<Expr>, Box<Expr>),
    Invalid,
}
pub const ZERO: Expr = Expr::Bas("0");
pub const ONE: Expr = Expr::Bas("1");
pub const UNIT: Expr = Expr::Bas("()");

fn step(e: &mut Box<Expr>) -> bool {
    match std::mem::replace(e.as_mut(), Expr::Invalid) {
        Expr::Invalid => {
            unreachable!("Evaluating empty expr")
        }
        expr @ (Expr::Bas(_) | Expr::Lam(_)) => {
            **e = expr;
            false
        }
        Expr::Ptr(Ptr(rc)) => {
            debug_assert_eq!(
                Rc::strong_count(&rc),
                1,
                "at this time Rc should have just one referent"
            );
            debug_assert!(rc.get().is_some(), "invalid deref before beta reduction");
            // we can take ownership of the value in the Rc since there are no other references by this point
            let Some(deref) = Rc::into_inner(rc).and_then(OnceCell::into_inner) else {
                unreachable!("Deref can't happen before beta reduction");
            };
            **e = *deref;
            true
        }
        Expr::App(mut f, mut v) => {
            if step(&mut f) || step(&mut v) {
                **e = Expr::App(f, v);
                true
            } else if let Expr::Lam(Lam(arg, body)) = *f {
                let _ = arg.0.set(v);
                **e = *body;
                true
            } else {
                // we're totally stuck, replace with the old app
                **e = Expr::App(f, v);
                false
            }
        }
    }
}

pub fn eval(e: Expr) -> Expr {
    let mut e = Box::new(e);
    println!("eval {e:?}");
    while step(&mut e) {
        println!("step {e:?}");
    }
    println!("Result: {e:?}");
    *e
}

/// ```compile_fail,E0373,E0505
/// use aptree::heaptree::{Expr,make_lam,make_app};
/// fn make_lam_cheat() -> Expr {
///     make_lam(|x| {
///         let mut cheat = Expr::Bas("0");
///         let lam = make_lam(|y| {
///             cheat = y;
///             x
///         });
///         make_app(lam, cheat)
///     })
/// }
/// ```
pub fn make_lam<F>(init: F) -> Expr
where
    F: FnOnce(Expr) -> Expr + 'static,
{
    let ptr = Ptr(Rc::new(OnceCell::new()));
    let body_ptr = Ptr(Rc::clone(&ptr.0));
    Expr::Lam(Lam(ptr, Box::new(init(Expr::Ptr(body_ptr)))))
}

pub fn make_app(f: Expr, v: Expr) -> Expr {
    Expr::App(Box::new(f), Box::new(v))
}

pub fn make_bas(c: &'static str) -> Expr {
    Expr::Bas(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ident() -> Expr {
        make_lam(|ptr| ptr)
    }
    fn make_const_fn() -> Expr {
        make_lam(|_ptr| UNIT)
    }
    fn make_lam_true() -> Expr {
        make_lam(|x| make_lam(|_y| x))
    }
    fn make_lam_false() -> Expr {
        make_lam(|_x| make_lam(|y| y))
    }

    #[test]
    fn t0() {
        let lam_const = make_const_fn();
        let app = make_app(lam_const, ONE);
        assert_eq!(UNIT, eval(app));
    }

    #[test]
    fn t1() {
        let lam_id = make_ident();
        let lam_const = make_const_fn();
        let app = make_app(make_app(lam_id, lam_const), ONE);
        assert_eq!(UNIT, eval(app));
    }

    #[test]
    fn t2() {
        let lam_true = make_lam_true();
        let app = make_app(make_app(lam_true, ZERO), ONE);
        assert_eq!(ZERO, eval(app));
    }

    #[test]
    fn t3() {
        let lam_false = make_lam_false();
        let app = make_app(make_app(lam_false, ZERO), ONE);
        assert_eq!(ONE, eval(app));
    }
}
