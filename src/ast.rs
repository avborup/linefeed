use std::rc::Rc;

use chumsky::span::SimpleSpan;

pub type Span = SimpleSpan;

#[derive(Clone, Debug)]
pub struct Spanned<T>(pub T, pub Span);

#[derive(Clone, Debug)]
pub enum TmpExpr<'src> {
    ParseError,
    Value(TmpValue<'src>),
    List(Vec<Spanned<Self>>),
    Index(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Local(&'src str),
    Let(&'src str, Box<Spanned<Self>>),
    Destructure(Vec<&'src str>, Box<Spanned<Self>>),
    Unary(UnaryOp, Box<Spanned<Self>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    Call(Box<Spanned<Self>>, Vec<Spanned<Self>>),
    MethodCall(Box<Spanned<Self>>, &'src str, Vec<Spanned<Self>>),
    If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Block(Box<Spanned<Self>>),
    Sequence(Vec<Spanned<Self>>),
    Return(Box<Spanned<Self>>),
    While(Box<Spanned<Self>>, Box<Spanned<Self>>),
    For(&'src str, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Break,
    Continue,
    ListComprehension(Box<Spanned<Self>>, &'src str, Box<Spanned<Self>>),
}

#[derive(Clone, Debug)]
pub enum TmpValue<'src> {
    Null,
    Bool(bool),
    Num(f64),
    Str(&'src str),
    Regex(&'src str),
    List(Vec<TmpValue<'src>>),
    Func(TmpFunc<'src>),
}

// An expression node in the AST. Children are spanned so we can generate useful runtime errors.
#[derive(Debug)]
pub enum Expr {
    ParseError,
    Value(Value),
    List(Vec<Spanned<Self>>),
    Index(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Local(String),
    Let(String, Box<Spanned<Self>>),
    Destructure(Vec<String>, Box<Spanned<Self>>),
    Unary(UnaryOp, Box<Spanned<Self>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    Call(Box<Spanned<Self>>, Vec<Spanned<Self>>),
    MethodCall(Box<Spanned<Self>>, String, Vec<Spanned<Self>>),
    If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Block(Box<Spanned<Self>>),
    Sequence(Vec<Spanned<Self>>),
    Return(Box<Spanned<Self>>),
    While(Box<Spanned<Self>>, Box<Spanned<Self>>),
    For(String, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Break,
    Continue,
    ListComprehension(Box<Spanned<Expr>>, String, Box<Spanned<Expr>>),
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Regex(String),
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
            Self::Regex(x) => write!(f, "{}", x),
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
    Mod,
    Or,
    And,
    Xor,
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Range,
}

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Neg,
    Not,
}

// A function node in the AST.
#[derive(Debug, Clone)]
pub struct Func {
    pub args: Vec<String>,
    pub body: Rc<Spanned<Expr>>,
}

#[derive(Debug, Clone)]
pub struct TmpFunc<'src> {
    pub args: Vec<&'src str>,
    pub body: Rc<Spanned<TmpExpr<'src>>>,
}

impl PartialEq for Func {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl PartialOrd for Func {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl<T> Spanned<T> {
    pub fn span(&self) -> Span {
        self.1
    }
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
