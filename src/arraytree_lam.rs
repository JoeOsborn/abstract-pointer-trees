pub const UNIT: &str = "()";
pub const ZERO: &str = "0";
pub const ONE: &str = "1";

#[allow(dead_code)]
#[derive(Debug)]
pub struct ExprRef(usize);
#[derive(Debug)]
pub struct ArgRef(usize);
#[derive(Debug)]
pub struct ExprDest(usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Expr {
    Bas(&'static str),
    Ptr(usize),
    Lam(usize, usize),
    App(usize, usize),
    Invalid,
}

#[derive(Debug)]
pub struct Program {
    exprs: Vec<Expr>,
}

impl Program {
    pub fn build(fun: impl FnOnce(&mut Program, ExprDest) -> ExprRef) -> Self {
        let mut out = Program {
            exprs: Vec::with_capacity(128),
        };
        out.exprs.push(Expr::Invalid);
        fun(&mut out, ExprDest(0));
        out
    }
    pub fn make_lam(
        &mut self,
        into: ExprDest,
        body: impl FnOnce(&mut Self, ArgRef, ExprDest) -> ExprRef,
    ) -> ExprRef {
        let lam_ref = into.0;
        assert_eq!(self.exprs[lam_ref], Expr::Invalid);
        self.exprs[lam_ref] = Expr::Lam(self.exprs.len(), self.exprs.len() + 1);
        let arg_ref = self.exprs.len();
        self.exprs.push(Expr::Invalid);
        let body_ref = self.exprs.len();
        self.exprs.push(Expr::Invalid);
        body(self, ArgRef(arg_ref), ExprDest(body_ref));
        ExprRef(lam_ref)
    }
    pub fn make_app(
        &mut self,
        into: ExprDest,
        fun: impl FnOnce(&mut Self, ExprDest) -> ExprRef,
        val: impl FnOnce(&mut Self, ExprDest) -> ExprRef,
    ) -> ExprRef {
        let app_ref = into.0;
        assert_eq!(self.exprs[app_ref], Expr::Invalid);
        self.exprs[app_ref] = Expr::App(self.exprs.len(), self.exprs.len() + 1);
        let f_ref = self.exprs.len();
        self.exprs.push(Expr::Invalid);
        let v_ref = self.exprs.len();
        self.exprs.push(Expr::Invalid);
        fun(self, ExprDest(f_ref));
        val(self, ExprDest(v_ref));
        ExprRef(app_ref)
    }
    pub fn make_const(&mut self, into: ExprDest, constant: &'static str) -> ExprRef {
        let const_ref = into.0;
        assert_eq!(self.exprs[const_ref], Expr::Invalid);
        self.exprs[const_ref] = Expr::Bas(constant);
        ExprRef(const_ref)
    }
    pub fn make_varref(&mut self, into: ExprDest, arg_ref: ArgRef) -> ExprRef {
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
            self.make_lam(e, |p, _ptr, e| p.make_const(e, UNIT))
        }
        fn make_ident(&mut self, e: ExprDest) -> ExprRef {
            self.make_lam(e, |p, ptr, e| p.make_varref(e, ptr))
        }
        fn make_lam_true(&mut self, e: ExprDest) -> ExprRef {
            self.make_lam(e, |p, x_ptr, e| {
                p.make_lam(e, |p, _y_ptr, e| p.make_varref(e, x_ptr))
            })
        }
        fn make_lam_false(&mut self, e: ExprDest) -> ExprRef {
            self.make_lam(e, |p, _x_ptr, e| {
                p.make_lam(e, |p, y_ptr, e| p.make_varref(e, y_ptr))
            })
        }
    }
    #[test]
    fn t0() {
        let mut app = Program::build(|p, e| {
            p.make_app(e, |p, e| p.make_const_fn(e), |p, e| p.make_const(e, ZERO))
        });
        assert_eq!(Some(UNIT), app.eval());
    }

    #[test]
    fn t1() {
        let mut app = Program::build(|p, e| {
            p.make_app(
                e,
                |p, e| p.make_app(e, |p, e| p.make_ident(e), |p, e| p.make_const_fn(e)),
                |p, e| p.make_const(e, ONE),
            )
        });
        assert_eq!(Some(UNIT), app.eval());
    }

    #[test]
    fn t2() {
        let mut app = Program::build(|p, e| {
            p.make_app(
                e,
                |p, e| p.make_app(e, |p, e| p.make_lam_true(e), |p, e| p.make_const(e, ZERO)),
                |p, e| p.make_const(e, ONE),
            )
        });
        assert_eq!(Some(ZERO), app.eval());
    }

    #[test]
    fn t3() {
        let mut app = Program::build(|p, e| {
            p.make_app(
                e,
                |p, e| p.make_app(e, |p, e| p.make_lam_false(e), |p, e| p.make_const(e, ZERO)),
                |p, e| p.make_const(e, ONE),
            )
        });
        assert_eq!(Some(ONE), app.eval());
    }
}
