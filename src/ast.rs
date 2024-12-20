use std::rc::Rc;

use crate::lexer::Span;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    List(Vec<Value>),
    Func(Rc<Func>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(x) => write!(f, "{}", x),
            Self::Num(x) => write!(f, "{}", x),
            Self::Str(x) => write!(f, "{}", x),
            Self::List(xs) => write!(
                f,
                "[{}]",
                xs.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Func(func) => write!(f, "<function, {} args>", func.args.len()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
}

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Neg,
    Not,
}

pub type Spanned<T> = (T, Span);

// An expression node in the AST. Children are spanned so we can generate useful runtime errors.
#[derive(Debug)]
pub enum Expr {
    Error,
    Value(Value),
    List(Vec<Spanned<Self>>),
    Local(String),
    Let(String, Box<Spanned<Self>>),
    Unary(UnaryOp, Box<Spanned<Self>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    Call(Box<Spanned<Self>>, Vec<Spanned<Self>>),
    If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Block(Box<Spanned<Self>>),
    Sequence(Vec<Spanned<Self>>),
    Print(Box<Spanned<Self>>),
}

// A function node in the AST.
#[derive(Debug, Clone)]
pub struct Func {
    pub args: Vec<String>,
    pub body: Rc<Spanned<Expr>>,
}

impl PartialEq for Func {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl Func {
    pub fn analyze_captures<T: AsRef<str>>(args: &[T], body: &Expr) -> Vec<String> {
        let mut captures = Vec::new();
        let mut stack = Vec::new();

        fn analyze_expr(expr: &Expr, stack: &mut Vec<String>, captures: &mut Vec<String>) {
            match expr {
                Expr::Error => unreachable!(),
                Expr::Value(Value::Func(sub_func)) => {
                    Func::analyze_captures(&sub_func.args, &sub_func.body.0)
                        .into_iter()
                        .for_each(|name| {
                            if !stack.contains(&name) {
                                captures.push(name);
                            }
                        });
                }
                Expr::Value(_) => {}
                Expr::List(items) => {
                    for item in items {
                        analyze_expr(&item.0, stack, captures);
                    }
                }
                Expr::Local(name) => {
                    if !stack.contains(name) {
                        captures.push(name.clone());
                    }
                }
                Expr::Let(local, val) => {
                    stack.push(local.clone());
                    analyze_expr(&val.0, stack, captures);
                    stack.pop();
                }
                Expr::Unary(_, a) => {
                    analyze_expr(&a.0, stack, captures);
                }
                Expr::Binary(a, _, b) => {
                    analyze_expr(&a.0, stack, captures);
                    analyze_expr(&b.0, stack, captures);
                }
                Expr::Call(func_expr, args) => {
                    analyze_expr(&func_expr.0, stack, captures);
                    for arg in args {
                        analyze_expr(&arg.0, stack, captures);
                    }
                }
                Expr::If(cond, a, b) => {
                    analyze_expr(&cond.0, stack, captures);
                    analyze_expr(&a.0, stack, captures);
                    analyze_expr(&b.0, stack, captures);
                }
                Expr::Block(inside) => {
                    analyze_expr(&inside.0, stack, captures);
                }
                Expr::Sequence(exprs) => {
                    for expr in exprs {
                        analyze_expr(&expr.0, stack, captures);
                    }
                }
                Expr::Print(a) => {
                    analyze_expr(&a.0, stack, captures);
                }
            }
        }

        analyze_expr(body, &mut stack, &mut captures);

        captures.retain(|name| !args.iter().any(|arg| arg.as_ref() == name));

        captures
    }
}
