use std::rc::Rc;

use chumsky::span::SimpleSpan;

use crate::vm::runtime_value::regex::RegexModifiers;

pub type Span = SimpleSpan;

#[derive(Clone, Debug)]
pub struct Spanned<T>(pub T, pub Span);

// An expression node in the AST. Children are spanned so we can generate useful errors.
#[derive(Debug, Clone)]
pub enum Expr<'src> {
    ParseError,
    Value(AstValue<'src>),
    List(Vec<Spanned<Self>>),
    Tuple(Vec<Spanned<Self>>),
    Map(Vec<(Spanned<Self>, Spanned<Self>)>),
    Index(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Local(&'src str),
    Assign(AssignmentTarget<'src>, Box<Spanned<Self>>),
    Unary(UnaryOp, Box<Spanned<Self>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    Call(Box<Spanned<Self>>, Vec<Spanned<Self>>),
    MethodCall(Box<Spanned<Self>>, &'src str, Vec<Spanned<Self>>),
    If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Block(Box<Spanned<Self>>),
    Sequence(Vec<Spanned<Self>>),
    Return(Box<Spanned<Self>>),
    While(Box<Spanned<Self>>, Box<Spanned<Self>>),
    For(
        AssignmentTarget<'src>,
        Box<Spanned<Self>>,
        Box<Spanned<Self>>,
    ),
    Break,
    Continue,
    ListComprehension(
        Box<Spanned<Self>>,
        AssignmentTarget<'src>,
        Box<Spanned<Self>>,
    ),
    Match(Box<Spanned<Self>>, Vec<(Spanned<Self>, Spanned<Self>)>),
}

#[derive(Clone, Debug)]
pub enum AstValue<'src> {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Regex(String, RegexModifiers),
    List(Vec<Self>),
    Tuple(Vec<Self>),
    Func(Func<'src>),
}

#[derive(Clone, Debug)]
pub enum AssignmentTarget<'src> {
    Local(&'src str),
    Destructure(Vec<AssignmentTarget<'src>>),
    Index(Box<Spanned<Expr<'src>>>, Box<Spanned<Expr<'src>>>),
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    DivFloor,
    Mod,
    Pow,
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
    In,
    BitwiseAnd,
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
