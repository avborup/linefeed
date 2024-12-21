use std::{collections::HashMap, io::Write};

use chumsky::error::Simple;

use crate::ast::{Expr, UnaryOp};

use crate::ast::{BinaryOp, Value};
use crate::lexer::{Span, Spanned};

pub struct Error {
    pub span: Span,
    pub msg: String,
}

impl Value {
    fn num(self, span: Span) -> Result<f64, Error> {
        if let Value::Num(x) = self {
            Ok(x)
        } else {
            Err(Error {
                span,
                msg: format!("'{self}' is not a number"),
            })
        }
    }

    fn bool(self, span: Span) -> Result<bool, Error> {
        match self {
            Value::Bool(x) => Ok(x),
            Value::Null => Ok(false),
            _ => Err(Error {
                span,
                msg: format!("'{self}' cannot be treated as a boolean"),
            }),
        }
    }
}

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
        self.eval_expr(expr)
            .map_err(|e| Simple::custom(e.span, e.msg))
    }

    pub fn eval_expr(&mut self, expr: &Spanned<Expr>) -> Result<Value, Error> {
        Ok(match &expr.0 {
            Expr::Error => unreachable!(), // Error expressions only get created by parser errors, so cannot exist in a valid AST

            Expr::Value(val) => val.clone(),

            Expr::List(items) => Value::List(
                items
                    .iter()
                    .map(|item| self.eval_expr(item))
                    .collect::<Result<_, _>>()?,
            ),

            Expr::Local(name) => self.vars.get(name).cloned().ok_or_else(|| Error {
                span: expr.1.clone(),
                msg: format!("No such variable '{}' in scope", name),
            })?,

            Expr::Let(local, val) => {
                let val = self.eval_expr(val)?;
                self.vars.set(local.clone(), val.clone());
                val // TODO: use Rc for values to avoid cloning
            }

            Expr::Unary(UnaryOp::Neg, a) => Value::Num(-self.eval_expr(a)?.num(a.1.clone())?),

            Expr::Unary(UnaryOp::Not, a) => Value::Bool(!self.eval_expr(a)?.bool(a.1.clone())?),

            Expr::Binary(a, op, b) => {
                let a_val = self.eval_expr(a)?;
                let b_val = self.eval_expr(b)?;

                match op {
                    BinaryOp::Add => Value::Num(a_val.num(a.1.clone())? + b_val.num(b.1.clone())?),
                    BinaryOp::Sub => Value::Num(a_val.num(a.1.clone())? - b_val.num(b.1.clone())?),
                    BinaryOp::Mul => Value::Num(a_val.num(a.1.clone())? * b_val.num(b.1.clone())?),
                    BinaryOp::Div => Value::Num(a_val.num(a.1.clone())? / b_val.num(b.1.clone())?),
                    BinaryOp::Eq => Value::Bool(a_val == b_val),
                    BinaryOp::NotEq => Value::Bool(a_val != b_val),
                }
            }

            Expr::Call(func_expr, args) => {
                let func_val = self.eval_expr(func_expr)?;

                let Value::Func(func) = func_val else {
                    return Err(Error {
                        span: func_expr.1.clone(),
                        msg: format!("'{func_val}' is not callable"),
                    });
                };

                if func.args.len() != args.len() {
                    return Err(Error {
                        span: func_expr.1.clone(),
                        msg: format!(
                        "function called with wrong number of arguments (expected {}, found {})",
                        func.args.len(),
                        args.len()
                    ),
                    });
                };

                self.vars.start_scope();
                for (name, arg) in func.args.iter().zip(args.iter()) {
                    let arg_val = self.eval_expr(arg)?;
                    self.vars.set_local(name.clone(), arg_val);
                }
                let res = self.eval_expr(&func.body)?;
                self.vars.pop_scope();
                res
            }

            Expr::If(cond, a, b) => {
                let c = self.eval_expr(cond)?;
                match c {
                    Value::Bool(true) => self.eval_expr(a)?,
                    Value::Bool(false) => self.eval_expr(b)?,
                    c => {
                        return Err(Error {
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
                .map(|expr| self.eval_expr(expr))
                .last()
                .unwrap_or(Ok(Value::Null))?,

            Expr::Print(a) => {
                let val = self.eval_expr(a)?;
                writeln!(self.stdout, "{val}").unwrap();
                val
            }
        })
    }
}

#[derive(Debug)]
pub struct VarStore {
    scopes: Vec<HashMap<String, Value>>,
}

impl Default for VarStore {
    fn default() -> Self {
        Self::new()
    }
}

impl VarStore {
    pub fn new() -> Self {
        let mut store = VarStore { scopes: Vec::new() };
        store.start_scope();
        store
    }

    fn start_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn get(&self, name: &str) -> Option<&Value> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    fn get_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(name))
    }

    fn set(&mut self, name: String, val: Value) {
        match self.get_mut(&name) {
            Some(existing) => *existing = val,
            None => self.set_local(name, val),
        }
    }

    fn set_local(&mut self, name: String, val: Value) {
        self.scopes.last_mut().unwrap().insert(name, val);
    }
}
