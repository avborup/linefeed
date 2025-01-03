use std::io::Write;

use chumsky::error::Simple;

use crate::{
    ast::{Expr, Span, Spanned, UnaryOp},
    scoped_map::ScopedMap,
};

use crate::ast::{BinaryOp, Value};

pub struct Interpreter<O: Write = std::io::Stdout, E: Write = std::io::Stderr> {
    pub stdout: O,
    pub stderr: E,
    pub vars: VarStore,
}

impl Default for Interpreter<std::io::Stdout, std::io::Stderr> {
    fn default() -> Self {
        Self::new_with_output(std::io::stdout(), std::io::stderr())
    }
}

impl<O: Write, E: Write> Interpreter<O, E> {
    pub fn new_with_output(stdout: O, stderr: E) -> Self {
        Self {
            stdout,
            stderr,
            vars: VarStore::new(),
        }
    }

    pub fn run(&mut self, expr: &Spanned<Expr>) -> Result<Value, Simple<String>> {
        self.eval_expr(expr).map_err(|e| e.simple())
    }

    pub fn eval_expr(&mut self, expr: &Spanned<Expr>) -> Result<Value, Error> {
        Ok(match &expr.0 {
            Expr::ParseError => unreachable!(), // Error expressions only get created by parser errors, so cannot exist in a valid AST

            Expr::Value(val) => val.clone(),

            Expr::List(items) => Value::List(
                items
                    .iter()
                    .map(|item| self.eval_expr(item))
                    .collect::<Result<_, _>>()?,
            ),

            Expr::Local(name) => self
                .vars
                .get(name)
                .ok_or_else(|| Error::Simple {
                    span: expr.1.clone(),
                    msg: format!("No such variable '{}' in scope", name),
                })?
                .inner()
                .clone(),

            Expr::Let(local, val) => {
                let val = self.eval_expr(val)?;
                self.vars.set(local.clone(), val.clone());
                val // TODO: use Rc for values to avoid cloning
            }

            Expr::Unary(UnaryOp::Neg, a) => Value::Num(-self.eval_num(a)?),
            Expr::Unary(UnaryOp::Not, a) => Value::Bool(!self.eval_bool(a)?),

            Expr::Binary(a, BinaryOp::Add, b) => Value::Num(self.eval_num(a)? + self.eval_num(b)?),
            Expr::Binary(a, BinaryOp::Sub, b) => Value::Num(self.eval_num(a)? - self.eval_num(b)?),
            Expr::Binary(a, BinaryOp::Mul, b) => Value::Num(self.eval_num(a)? * self.eval_num(b)?),
            Expr::Binary(a, BinaryOp::Div, b) => Value::Num(self.eval_num(a)? / self.eval_num(b)?),

            Expr::Binary(a, BinaryOp::Eq, b) => {
                Value::Bool(self.eval_expr(a)? == self.eval_expr(b)?)
            }
            Expr::Binary(a, BinaryOp::NotEq, b) => {
                Value::Bool(self.eval_expr(a)? != self.eval_expr(b)?)
            }
            Expr::Binary(a, BinaryOp::Less, b) => {
                Value::Bool(self.eval_expr(a)? < self.eval_expr(b)?)
            }
            Expr::Binary(a, BinaryOp::LessEq, b) => {
                Value::Bool(self.eval_expr(a)? <= self.eval_expr(b)?)
            }
            Expr::Binary(a, BinaryOp::Greater, b) => {
                Value::Bool(self.eval_expr(a)? > self.eval_expr(b)?)
            }
            Expr::Binary(a, BinaryOp::GreaterEq, b) => {
                Value::Bool(self.eval_expr(a)? >= self.eval_expr(b)?)
            }

            Expr::Binary(a, BinaryOp::Or, b) => {
                Value::Bool(self.eval_bool(a)? || self.eval_bool(b)?)
            }
            Expr::Binary(a, BinaryOp::And, b) => {
                Value::Bool(self.eval_bool(a)? && self.eval_bool(b)?)
            }

            Expr::Return(val) => {
                return Err(Error::Return {
                    val: self.eval_expr(val)?,
                    span: expr.1.clone(),
                })
            }

            Expr::Call(func_expr, args) => {
                let func_val = self.eval_expr(func_expr)?;

                let Value::Func(func) = func_val else {
                    return Err(Error::Simple {
                        span: func_expr.1.clone(),
                        msg: format!("'{func_val}' is not callable"),
                    });
                };

                if func.args.len() != args.len() {
                    return Err(Error::Simple {
                        span: func_expr.1.clone(),
                        msg: format!(
                            "function called with wrong number of arguments (expected {}, found {})",
                            func.args.len(),
                            args.len()
                        )
                    });
                };

                self.vars.start_scope();
                for (name, arg) in func.args.iter().zip(args.iter()) {
                    let arg_val = self.eval_expr(arg)?;
                    self.vars.set_local(name.clone(), arg_val);
                }
                let res = self.eval_expr(&func.body);
                self.vars.pop_scope();

                match res {
                    Ok(val) => val,
                    Err(Error::Return { val, .. }) => val,
                    Err(e) => return Err(e),
                }
            }

            Expr::If(cond, a, b) => {
                let c = self.eval_expr(cond)?;
                match c {
                    Value::Bool(true) => self.eval_expr(a)?,
                    Value::Bool(false) => self.eval_expr(b)?,
                    c => {
                        return Err(Error::Simple {
                            span: cond.1.clone(),
                            msg: format!("Conditions must be booleans, found '{:?}'", c),
                        })
                    }
                }
            }

            Expr::Block(sub_expr) => {
                self.vars.start_scope();
                let res = self.eval_expr(sub_expr)?;
                self.vars.pop_scope();
                res
            }

            Expr::Sequence(exprs) => exprs
                .iter()
                .try_fold(Value::Null, |_, expr| self.eval_expr(expr))?,

            Expr::Print(a) => {
                let val = self.eval_expr(a)?;
                writeln!(self.stdout, "{val}").unwrap();
                val
            }

            _ => unimplemented!("{:?}", expr.0),
        })
    }

    pub fn eval_num(&mut self, expr: &Spanned<Expr>) -> Result<f64, Error> {
        self.eval_expr(expr)?.num(expr.1.clone())
    }

    pub fn eval_bool(&mut self, expr: &Spanned<Expr>) -> Result<bool, Error> {
        self.eval_expr(expr)?.bool(expr.1.clone())
    }
}

#[derive(Debug)]
pub enum Error {
    Simple { span: Span, msg: String },
    Return { span: Span, val: Value },
}

impl Error {
    pub fn simple(self) -> Simple<String> {
        match self {
            Error::Simple { span, msg } => Simple::custom(span, msg),
            Error::Return { span, .. } => {
                Simple::custom(span, "illegal return outside of function")
            }
        }
    }
}

impl Value {
    fn num(self, span: Span) -> Result<f64, Error> {
        if let Value::Num(x) = self {
            Ok(x)
        } else {
            Err(Error::Simple {
                span,
                msg: format!("'{self}' is not a number"),
            })
        }
    }

    fn bool(self, span: Span) -> Result<bool, Error> {
        match self {
            Value::Bool(x) => Ok(x),
            Value::Null => Ok(false),
            Value::Num(x) => Ok(x != 0.0),
            Value::List(items) => Ok(!items.is_empty()),
            Value::Str(s) => Ok(!s.is_empty()),
            _ => Err(Error::Simple {
                span,
                msg: format!("'{self}' cannot be treated as a boolean"),
            }),
        }
    }
}

pub type VarStore = ScopedMap<String, Value>;
