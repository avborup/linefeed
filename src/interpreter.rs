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

pub fn eval_expr(expr: &Spanned<Expr>, stack: &mut Vec<(String, Value)>) -> Result<Value, Error> {
    Ok(match &expr.0 {
        Expr::Error => unreachable!(), // Error expressions only get created by parser errors, so cannot exist in a valid AST
        Expr::Value(val) => val.clone(),
        Expr::List(items) => Value::List(
            items
                .iter()
                .map(|item| eval_expr(item, stack))
                .collect::<Result<_, _>>()?,
        ),
        Expr::Local(name) => stack
            .iter()
            .rev()
            .find(|(l, _)| l == name)
            .map(|(_, v)| v.clone())
            .ok_or_else(|| Error {
                span: expr.1.clone(),
                msg: format!("No such variable '{}' in scope", name),
            })?,
        Expr::Let(local, val, body) => {
            let val = eval_expr(val, stack)?;
            stack.push((local.clone(), val));
            let res = eval_expr(body, stack)?;
            stack.pop();
            res
        }
        Expr::Then(a, b) => {
            eval_expr(a, stack)?;
            eval_expr(b, stack)?
        }
        Expr::Unary(UnaryOp::Neg, a) => Value::Num(-eval_expr(a, stack)?.num(a.1.clone())?),
        Expr::Unary(UnaryOp::Not, a) => Value::Bool(!eval_expr(a, stack)?.bool(a.1.clone())?),
        Expr::Binary(a, BinaryOp::Add, b) => Value::Num(
            eval_expr(a, stack)?.num(a.1.clone())? + eval_expr(b, stack)?.num(b.1.clone())?,
        ),
        Expr::Binary(a, BinaryOp::Sub, b) => Value::Num(
            eval_expr(a, stack)?.num(a.1.clone())? - eval_expr(b, stack)?.num(b.1.clone())?,
        ),
        Expr::Binary(a, BinaryOp::Mul, b) => Value::Num(
            eval_expr(a, stack)?.num(a.1.clone())? * eval_expr(b, stack)?.num(b.1.clone())?,
        ),
        Expr::Binary(a, BinaryOp::Div, b) => Value::Num(
            eval_expr(a, stack)?.num(a.1.clone())? / eval_expr(b, stack)?.num(b.1.clone())?,
        ),
        Expr::Binary(a, BinaryOp::Eq, b) => {
            Value::Bool(eval_expr(a, stack)? == eval_expr(b, stack)?)
        }
        Expr::Binary(a, BinaryOp::NotEq, b) => {
            Value::Bool(eval_expr(a, stack)? != eval_expr(b, stack)?)
        }
        Expr::Call(func_expr, args) => {
            let func_val = eval_expr(func_expr, stack)?;

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

            // TODO: re-add recursion here
            let mut stack = func
                .args
                .iter()
                .zip(args.iter())
                .map(|(name, arg)| Ok((name.clone(), eval_expr(arg, stack)?)))
                .collect::<Result<Vec<_>, _>>()?;

            eval_expr(&func.body, &mut stack)?
        }
        Expr::If(cond, a, b) => {
            let c = eval_expr(cond, stack)?;
            match c {
                Value::Bool(true) => eval_expr(a, stack)?,
                Value::Bool(false) => eval_expr(b, stack)?,
                c => {
                    return Err(Error {
                        span: cond.1.clone(),
                        msg: format!("Conditions must be booleans, found '{:?}'", c),
                    })
                }
            }
        }
        Expr::Print(a) => {
            let val = eval_expr(a, stack)?;
            println!("{}", val);
            val
        }
    })
}
