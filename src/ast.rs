use std::rc::Rc;

use chumsky::span::SimpleSpan;

pub type Span = SimpleSpan;

#[derive(Clone, Debug)]
pub struct Spanned<T>(pub T, pub Span);

// An expression node in the AST. Children are spanned so we can generate useful errors.
#[derive(Debug)]
pub enum Expr<'src> {
    ParseError,
    Value(AstValue<'src>),
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
pub enum AstValue<'src> {
    Null,
    Bool(bool),
    Num(f64),
    Str(&'src str),
    Regex(&'src str),
    List(Vec<Self>),
    Func(Func<'src>),
}

impl<'src> std::fmt::Display for AstValue<'src> {
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

#[derive(Debug, Clone)]
pub struct Func<'src> {
    pub args: Vec<&'src str>,
    pub body: Rc<Spanned<Expr<'src>>>,
}

impl<'src> PartialEq for Func<'src> {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl<'src> PartialOrd for Func<'src> {
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
