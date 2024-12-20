use std::collections::HashMap;

use crate::ast::{Expr, Func, UnaryOp};

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

pub fn eval_expr(expr: &Spanned<Expr>, vars: &mut VarStore) -> Result<Value, Error> {
    Ok(match &expr.0 {
        Expr::Error => unreachable!(), // Error expressions only get created by parser errors, so cannot exist in a valid AST
        Expr::Value(val) => val.clone(),
        Expr::List(items) => Value::List(
            items
                .iter()
                .map(|item| eval_expr(item, vars))
                .collect::<Result<_, _>>()?,
        ),
        Expr::Local(name) => vars.get(name).cloned().ok_or_else(|| Error {
            span: expr.1.clone(),
            msg: format!("No such variable '{}' in scope", name),
        })?,
        Expr::Let(local, val) => {
            let val = eval_expr(val, vars)?;
            vars.set(local.clone(), val.clone());
            val // TODO: use Rc for values to avoid cloning
        }
        Expr::Unary(UnaryOp::Neg, a) => Value::Num(-eval_expr(a, vars)?.num(a.1.clone())?),
        Expr::Unary(UnaryOp::Not, a) => Value::Bool(!eval_expr(a, vars)?.bool(a.1.clone())?),
        Expr::Binary(a, BinaryOp::Add, b) => Value::Num(
            eval_expr(a, vars)?.num(a.1.clone())? + eval_expr(b, vars)?.num(b.1.clone())?,
        ),
        Expr::Binary(a, BinaryOp::Sub, b) => Value::Num(
            eval_expr(a, vars)?.num(a.1.clone())? - eval_expr(b, vars)?.num(b.1.clone())?,
        ),
        Expr::Binary(a, BinaryOp::Mul, b) => Value::Num(
            eval_expr(a, vars)?.num(a.1.clone())? * eval_expr(b, vars)?.num(b.1.clone())?,
        ),
        Expr::Binary(a, BinaryOp::Div, b) => Value::Num(
            eval_expr(a, vars)?.num(a.1.clone())? / eval_expr(b, vars)?.num(b.1.clone())?,
        ),
        Expr::Binary(a, BinaryOp::Eq, b) => Value::Bool(eval_expr(a, vars)? == eval_expr(b, vars)?),
        Expr::Binary(a, BinaryOp::NotEq, b) => {
            Value::Bool(eval_expr(a, vars)? != eval_expr(b, vars)?)
        }
        Expr::Call(func_expr, args) => {
            let func_val = eval_expr(func_expr, vars)?;

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

            let mut func_scope = VarStore::new();

            for required_var in Func::analyze_captures(&func.args, &func.body.0) {
                if let Some(val) = vars.get(&required_var) {
                    func_scope.set(required_var, val.clone());
                }
            }

            for (name, arg) in func.args.iter().zip(args.iter()) {
                func_scope.set(name.clone(), eval_expr(arg, vars)?);
            }

            eval_expr(&func.body, &mut func_scope)?
        }
        Expr::If(cond, a, b) => {
            let c = eval_expr(cond, vars)?;
            match c {
                Value::Bool(true) => eval_expr(a, vars)?,
                Value::Bool(false) => eval_expr(b, vars)?,
                c => {
                    return Err(Error {
                        span: cond.1.clone(),
                        msg: format!("Conditions must be booleans, found '{:?}'", c),
                    })
                }
            }
        }
        Expr::Block(sub_expr) => {
            vars.start_scope();
            let res = eval_expr(sub_expr, vars)?;
            vars.pop_scope();
            res
        }
        Expr::Sequence(exprs) => exprs
            .iter()
            .map(|expr| eval_expr(expr, vars))
            .last()
            .unwrap_or(Ok(Value::Null))?,
        Expr::Print(a) => {
            let val = eval_expr(a, vars)?;
            println!("{}", val);
            val
        }
    })
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

    fn set(&mut self, name: String, val: Value) {
        self.scopes
            .last_mut()
            .expect("No scope to set variable in")
            .insert(name, val);
    }
}
