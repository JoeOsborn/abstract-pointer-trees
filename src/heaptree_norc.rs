use std::cell::Cell;

pub struct Ptr<'prg>(&'prg Cell<Expr<'prg>>);
pub struct Lam<'prg>(Ptr<'prg>, Box<Expr<'prg>>);

pub enum Expr<'prg> {
    Ptr(Ptr<'prg>),
    Bas(&'static str),
    Lam(Lam<'prg>),
    App(Box<Expr<'prg>>, Box<Expr<'prg>>),
    Invalid,
}

impl<'prg> std::fmt::Display for Expr<'prg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Ptr(Ptr(ptr)) => write!(f, "@{ptr:p}"),
            Expr::Bas(b) => write!(f, "{b}"),
            Expr::Lam(Lam(Ptr(ptr), body)) => write!(f, "\\{ptr:p} -> {body}"),
            Expr::App(fun, val) => write!(f, "({fun} {val})"),
            Expr::Invalid => write!(f, "NUL"),
        }
    }
}

pub const ZERO: &str = "0";
pub const ONE: &str = "1";
pub const UNIT: &str = "()";

fn step(e: &mut Box<Expr<'_>>) -> bool {
    match std::mem::replace(e.as_mut(), Expr::Invalid) {
        Expr::Invalid => {
            unreachable!("Evaluating empty expr")
        }
        expr @ (Expr::Bas(_) | Expr::Lam(_)) => {
            **e = expr;
            false
        }
        Expr::Ptr(Ptr(cell)) => {
            let deref = cell.replace(Expr::Invalid);
            debug_assert!(!matches!(deref, Expr::Invalid));
            **e = deref;
            true
        }
        Expr::App(mut f, mut v) => {
            if step(&mut f) || step(&mut v) {
                **e = Expr::App(f, v);
                true
            } else if let Expr::Lam(Lam(Ptr(cell), body)) = *f {
                let _old = cell.replace(*v);
                debug_assert!(matches!(_old, Expr::Invalid));
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

pub fn eval(e: Expr<'_>) -> Expr<'_> {
    let mut e = Box::new(e);
    println!("eval {e}");
    while step(&mut e) {
        println!("step {e}");
    }
    println!("Result: {e}");
    *e
}

pub fn make_lam<'a, 'b>(args: &'b Args<'a>, init: impl FnOnce(Expr<'a>) -> Expr<'a>) -> Expr<'a>
where
    'b: 'a,
{
    let idx = args.1.take();
    args.1.set(idx + 1);
    let cell = &args.0[idx];
    let ptr = Ptr(cell);
    let body_ptr = Ptr(cell);
    Expr::Lam(Lam(ptr, Box::new(init(Expr::Ptr(body_ptr)))))
}

pub fn make_app<'prg>(f: Expr<'prg>, v: Expr<'prg>) -> Expr<'prg> {
    Expr::App(Box::new(f), Box::new(v))
}

pub fn make_bas<'prg>(c: &'static str) -> Expr<'prg> {
    Expr::Bas(c)
}

pub struct Args<'prg>(Vec<Cell<Expr<'prg>>>, std::cell::Cell<usize>);
impl<'prg> Args<'prg> {
    pub fn with_capacity(cap: usize) -> Self {
        Self(
            Vec::from_iter((0..cap).map(|_| Cell::new(Expr::Invalid))),
            std::cell::Cell::new(0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ident<'a>(args: &'a Args<'a>) -> Expr<'a> {
        make_lam(args, |ptr| ptr)
    }
    fn make_const_fn<'a>(args: &'a Args<'a>) -> Expr<'a> {
        make_lam(args, |_ptr| Expr::Bas(UNIT))
    }
    fn make_lam_true<'a>(args: &'a Args<'a>) -> Expr<'a> {
        make_lam(args, |x| make_lam(args, |_y| x))
    }
    fn make_lam_false<'a>(args: &'a Args<'a>) -> Expr<'a> {
        make_lam(args, |_x| make_lam(args, |y| y))
    }

    #[test]
    fn t0() {
        let args = Args::with_capacity(128);
        let lam_const = make_const_fn(&args);
        let app = make_app(lam_const, Expr::Bas(ONE));
        assert!(matches!(eval(app), Expr::Bas(UNIT)));
    }

    #[test]
    fn t1() {
        let args = Args::with_capacity(128);
        let lam_id = make_ident(&args);
        let lam_const = make_const_fn(&args);
        let app = make_app(make_app(lam_id, lam_const), Expr::Bas(ONE));
        assert!(matches!(eval(app), Expr::Bas(UNIT)));
    }

    #[test]
    fn t2() {
        let args = Args::with_capacity(128);
        let lam_true = make_lam_true(&args);
        let app = make_app(make_app(lam_true, Expr::Bas(ZERO)), Expr::Bas(ONE));
        assert!(matches!(eval(app), Expr::Bas(ZERO)));
    }

    #[test]
    fn t3() {
        let args = Args::with_capacity(128);
        let lam_false = make_lam_false(&args);
        let app = make_app(make_app(lam_false, Expr::Bas(ZERO)), Expr::Bas(ONE));
        assert!(matches!(eval(app), Expr::Bas(ONE)));
    }
}
