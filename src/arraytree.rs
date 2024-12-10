pub const ZERO: &str = "0";
pub const ONE: &str = "1";
pub const UNIT: &str = "()";

#[allow(dead_code)]
#[derive(Debug)]
pub struct ExprRef(usize);
#[derive(Debug)]
pub struct ArgRef(usize);
#[derive(Debug)]
pub struct ExprDest(usize);

#[derive(Debug)]
pub struct Program {
    exprs: Vec<Expr>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Expr {
    Bas(&'static str),
    Ptr(usize),
    Lam(usize, usize),
    App(usize, usize),
    Invalid,
}

impl Program {
    pub fn build() -> (Self, ExprDest) {
        let mut p = Self {
            exprs: Vec::with_capacity(128),
        };
        p.exprs.push(Expr::Invalid);
        (p, ExprDest(0))
    }
    pub fn make_const(&mut self, into: ExprDest, c: &'static str) -> ExprRef {
        let const_ref = into.0;
        assert_eq!(self.exprs[const_ref], Expr::Invalid);
        self.exprs[const_ref] = Expr::Bas(c);
        ExprRef(const_ref)
    }
    pub fn make_lam(&mut self, into: ExprDest) -> (ExprRef, ArgRef, ExprDest) {
        let lam_ref = into.0;
        assert_eq!(self.exprs[lam_ref], Expr::Invalid);
        self.exprs[lam_ref] = Expr::Lam(self.exprs.len(), self.exprs.len() + 1);
        let arg_ref = self.exprs.len();
        self.exprs.push(Expr::Invalid);
        let body_ref = self.exprs.len();
        self.exprs.push(Expr::Invalid);
        (ExprRef(lam_ref), ArgRef(arg_ref), ExprDest(body_ref))
    }
    pub fn make_app(&mut self, into: ExprDest) -> (ExprRef, ExprDest, ExprDest) {
        let app_ref = into.0;
        assert_eq!(self.exprs[app_ref], Expr::Invalid);
        self.exprs[app_ref] = Expr::App(self.exprs.len(), self.exprs.len() + 1);
        let f_ref = self.exprs.len();
        self.exprs.push(Expr::Invalid);
        let v_ref = self.exprs.len();
        self.exprs.push(Expr::Invalid);
        (ExprRef(app_ref), ExprDest(f_ref), ExprDest(v_ref))
    }
    pub fn make_deref(&mut self, into: ExprDest, arg_ref: ArgRef) -> ExprRef {
        let deref = into.0;
        assert_eq!(self.exprs[deref], Expr::Invalid);
        self.exprs[deref] = Expr::Ptr(arg_ref.0);
        ExprRef(deref)
    }
    fn step(&mut self, expr_idx: usize) -> bool {
        let expr = self.exprs[expr_idx];
        match expr {
            Expr::Invalid => unreachable!("Not fully initialized program!"),
            Expr::Bas(_) | Expr::Lam(_, _) => false,
            Expr::Ptr(target) => {
                // deref this expr to self.exprs[target]
                self.exprs[expr_idx] = self.exprs[target];

                self.exprs[target] = Expr::Invalid;
                true
            }
            Expr::App(f, v) => {
                self.step(f) || self.step(v) || {
                    let Expr::App(f, v) = self.exprs[expr_idx] else {
                        unreachable!("we already know expr is app");
                    };
                    if let Expr::Lam(arg, body) = self.exprs[f] {
                        // beta reduction: set arg to v, replace expr with body
                        self.exprs[arg] = self.exprs[v];
                        self.exprs[expr_idx] = self.exprs[body];

                        self.exprs[v] = Expr::Invalid;
                        self.exprs[body] = Expr::Invalid;
                        true
                    } else {
                        panic!("stuck, f value is not a function {:?}", self.exprs[f]);
                    }
                }
            }
        }
    }
    pub fn eval(&mut self) -> Option<&'static str> {
        println!("eval {self:?}");
        while self.step(0) {
            println!("step {self:?}");
        }
        println!("result {self:?}");
        if let Expr::Bas(result) = self.exprs[0] {
            Some(result)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    trait ProgramExt {
        fn make_const_fn(&mut self, e: ExprDest) -> ExprRef;
        fn make_ident(&mut self, e: ExprDest) -> ExprRef;
        fn make_lam_true(&mut self, e: ExprDest) -> ExprRef;
        fn make_lam_false(&mut self, e: ExprDest) -> ExprRef;
    }
    impl ProgramExt for Program {
        fn make_const_fn(&mut self, e: ExprDest) -> ExprRef {
            let (lam, _arg, body) = self.make_lam(e);
            let _ = self.make_const(body, UNIT);
            lam
        }
        fn make_ident(&mut self, e: ExprDest) -> ExprRef {
            let (lam, arg, body) = self.make_lam(e);
            let _ = self.make_deref(body, arg);
            lam
        }
        fn make_lam_true(&mut self, e: ExprDest) -> ExprRef {
            let (x_lam, x_arg, x_body) = self.make_lam(e);
            let (_y_lam, _y_arg, y_body) = self.make_lam(x_body);
            let _ = self.make_deref(y_body, x_arg);
            x_lam
        }
        fn make_lam_false(&mut self, e: ExprDest) -> ExprRef {
            let (x_lam, _x_arg, x_body) = self.make_lam(e);
            let (_y_lam, y_arg, y_body) = self.make_lam(x_body);
            let _ = self.make_deref(y_body, y_arg);
            x_lam
        }
    }
    #[test]
    fn t0() {
        let (mut prg, start) = Program::build();
        let (_app, f, v) = prg.make_app(start);
        let _ = prg.make_const_fn(f);
        let _ = prg.make_const(v, ONE);
        assert_eq!(Some(UNIT), prg.eval());
    }

    #[test]
    fn t1() {
        let (mut prg, start) = Program::build();
        let (_app, f, v) = prg.make_app(start);
        let (_f, ff, fv) = prg.make_app(f);
        let _ = prg.make_ident(ff);
        let _ = prg.make_const_fn(fv);
        let _ = prg.make_const(v, ONE);
        assert_eq!(Some(UNIT), prg.eval());
    }

    #[test]
    fn t2() {
        let (mut prg, start) = Program::build();
        let (_app, f, v) = prg.make_app(start);
        let (_f, ff, fv) = prg.make_app(f);
        let _ = prg.make_lam_true(ff);
        let _ = prg.make_const(fv, ZERO);
        let _ = prg.make_const(v, ONE);
        assert_eq!(Some(ZERO), prg.eval());
    }

    #[test]
    fn t3() {
        let (mut prg, start) = Program::build();
        let (_app, f, v) = prg.make_app(start);
        let (_f, ff, fv) = prg.make_app(f);
        let _ = prg.make_lam_false(ff);
        let _ = prg.make_const(fv, ZERO);
        let _ = prg.make_const(v, ONE);
        assert_eq!(Some(ONE), prg.eval());
    }
}
