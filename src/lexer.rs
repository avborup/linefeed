use chumsky::prelude::*;
use std::fmt;

use crate::ast::{Span, Spanned};

#[derive(Clone, Debug, PartialEq)]
pub enum TmpToken<'src> {
    Null,
    Bool(bool),
    Num(f64),
    Str(&'src str),
    Op(&'src str),
    Ctrl(char),
    Ident(&'src str),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Regex(String),
    Op(String),
    Ctrl(char),
    Ident(String),
    If,
    Else,
    Or,
    And,
    Not,
    Xor,
    Fn,
    Return,
    Unless,
    While,
    For,
    In,
    Break,
    Continue,
}

impl Token {
    pub fn op(s: impl Into<String>) -> Token {
        Token::Op(s.into())
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Null => write!(f, "null"),
            Token::Bool(x) => write!(f, "{}", x),
            Token::Num(n) => write!(f, "{}", n),
            Token::Str(s) => write!(f, "{}", s),
            Token::Regex(r) => write!(f, "{}", r),
            Token::Op(s) => write!(f, "{}", s),
            Token::Ctrl(c) => write!(f, "{}", c),
            Token::Ident(s) => write!(f, "{}", s),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Or => write!(f, "or"),
            Token::Not => write!(f, "not"),
            Token::And => write!(f, "and"),
            Token::Xor => write!(f, "xor"),
            Token::Fn => write!(f, "fn"),
            Token::Return => write!(f, "return"),
            Token::Unless => write!(f, "unless"),
            Token::While => write!(f, "while"),
            Token::For => write!(f, "for"),
            Token::In => write!(f, "in"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
        }
    }
}

pub fn lexer<'src>(
) -> impl Parser<'src, &'src str, Vec<Spanned<TmpToken<'src>>>, extra::Err<Rich<'src, char, Span>>>
{
    let num = text::int(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .from_str()
        .unwrapped()
        .map(TmpToken::Num);

    let comment = just("#")
        .then(any().and_is(just('\n').not()).repeated())
        .padded();

    let token = num;

    token
        .map_with(|tok, e| Spanned(tok, e.span()))
        .padded_by(comment.repeated())
        .padded()
        // If we encounter an error, skip and attempt to lex the next character as a token instead
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
    // let num = text::int(10)
    //     .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
    //     .collect::<String>()
    //     .map(Token::Num);
    //
    // let raw_str = just("r\"")
    //     .ignore_then(filter(|c| *c != '"').repeated())
    //     .then_ignore(just('"'))
    //     .collect::<String>()
    //     .map(Token::Str);
    //
    // let simple_str = just('"')
    //     .ignore_then(choice((just(r"\n").to('\n'), filter(|c| *c != '"'))).repeated())
    //     .then_ignore(just('"'))
    //     .collect::<String>()
    //     .map(Token::Str);
    //
    // let regex_str = just('/')
    //     .ignore_then(filter(|c| *c != '/').repeated())
    //     .then_ignore(just('/'))
    //     .collect::<String>()
    //     .map(Token::Regex);
    //
    // let str_ = raw_str.or(simple_str);
    //
    // let range = just("..").to(Token::op(".."));
    //
    // let op = one_of("+-*/!=<>%")
    //     .repeated()
    //     .at_least(1)
    //     .collect::<String>()
    //     .map(Token::Op);
    //
    // let ctrl = one_of("()[]{};,|.").map(Token::Ctrl);
    //
    // let ident = text::ident().map(|ident: String| match ident.as_str() {
    //     "if" => Token::If,
    //     "else" => Token::Else,
    //     "true" => Token::Bool(true),
    //     "false" => Token::Bool(false),
    //     "or" => Token::Or,
    //     "and" => Token::And,
    //     "not" => Token::Not,
    //     "xor" => Token::Xor,
    //     "null" => Token::Null,
    //     "fn" => Token::Fn,
    //     "return" => Token::Return,
    //     "unless" => Token::Unless,
    //     "while" => Token::While,
    //     "for" => Token::For,
    //     "in" => Token::In,
    //     "break" => Token::Break,
    //     "continue" => Token::Continue,
    //     _ => Token::Ident(ident),
    // });
    //
    // let token = num
    //     .or(str_)
    //     .or(regex_str)
    //     .or(range)
    //     .or(op)
    //     .or(ctrl)
    //     .or(ident)
    //     .recover_with(skip_then_retry_until([]));
    //
    // let comment = just('#').then(take_until(just('\n'))).padded();
    //
    // token
    //     .map_with_span(|tok, span| (tok, span))
    //     .padded_by(comment.repeated())
    //     .padded()
    //     .repeated()
}
