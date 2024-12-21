use std::rc::Rc;

use chumsky::prelude::*;

use crate::ast::{BinaryOp, Expr, Func, Span, Spanned, UnaryOp, Value};
use crate::lexer::Token;

pub fn expr_parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        // Blocks are expressions but delimited with braces
        let block = expr
            .clone()
            .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}')))
            .map_with_span(|expr, span| Spanned(Expr::Block(Box::new(expr)), span))
            // Attempt to recover anything that looks like a block but contains errors
            .recover_with(nested_delimiters(
                Token::Ctrl('{'),
                Token::Ctrl('}'),
                [
                    (Token::Ctrl('('), Token::Ctrl(')')),
                    (Token::Ctrl('['), Token::Ctrl(']')),
                ],
                |span| Spanned(Expr::Error, span),
            ));

        let if_ = recursive(|if_| {
            just(Token::If)
                .ignore_then(expr.clone())
                .then(block.clone())
                .then(
                    just(Token::Else)
                        .ignore_then(block.clone().or(if_))
                        .or_not(),
                )
                .map_with_span(|((cond, a), b), span: Span| {
                    Spanned(
                        Expr::If(
                            Box::new(cond),
                            Box::new(a),
                            Box::new(match b {
                                Some(b) => b,
                                // If an `if` expression has no trailing `else` block, we magic up one that just produces null
                                None => Spanned(Expr::Value(Value::Null), span.clone()),
                            }),
                        ),
                        span,
                    )
                })
        });

        // Both blocks and `if` are 'block expressions' and can appear in the place of statements
        let block_expr = block.clone().or(if_).labelled("block");

        let raw_expr = recursive(|raw_expr| {
            let val = select! {
                Token::Null => Expr::Value(Value::Null),
                Token::Bool(x) => Expr::Value(Value::Bool(x)),
                Token::Num(n) => Expr::Value(Value::Num(n.parse().unwrap())),
                Token::Str(s) => Expr::Value(Value::Str(s)),
            }
            .labelled("value");

            let ident = select! { Token::Ident(ident) => ident.clone() }.labelled("identifier");

            // A list of expressions
            let items = expr
                .clone()
                .separated_by(just(Token::Ctrl(',')))
                .allow_trailing();

            // Argument lists are just identifiers separated by commas, surrounded by parentheses
            let args = ident
                .separated_by(just(Token::Ctrl(',')))
                .allow_trailing()
                .delimited_by(just(Token::Ctrl('|')), just(Token::Ctrl('|')))
                .labelled("function args");

            let func = args
                .then(
                    block_expr
                        .clone()
                        // Attempt to recover anything that looks like a function body but contains errors
                        .recover_with(nested_delimiters(
                            Token::Ctrl('{'),
                            Token::Ctrl('}'),
                            [
                                (Token::Ctrl('('), Token::Ctrl(')')),
                                (Token::Ctrl('['), Token::Ctrl(']')),
                            ],
                            |span| Spanned(Expr::Error, span),
                        ))
                        .or(raw_expr.clone()),
                )
                .map(|(args, body)| {
                    Expr::Value(Value::Func(Rc::new(Func {
                        args,
                        body: Rc::new(body),
                    })))
                })
                .labelled("function");

            // Variable assignment
            let let_ = ident
                .then_ignore(just(Token::Op("=".to_string())))
                .then(raw_expr.clone().or(block_expr.clone()))
                .map(|(name, val)| Expr::Let(name, Box::new(val)));

            let list = items
                .clone()
                .delimited_by(just(Token::Ctrl('[')), just(Token::Ctrl(']')))
                .map(Expr::List);

            // 'Atoms' are expressions that contain no ambiguity
            let atom = val
                .or(func)
                .or(let_)
                .or(ident.map(Expr::Local))
                .or(list)
                // In Nano Rust, `print` is just a keyword, just like Python 2, for simplicity
                .or(just(Token::Print)
                    .ignore_then(
                        expr.clone()
                            .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')'))),
                    )
                    .map(|expr| Expr::Print(Box::new(expr))))
                .map_with_span(Spanned)
                // Atoms can also just be normal expressions, but surrounded with parentheses
                .or(expr
                    .clone()
                    .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')'))))
                // Attempt to recover anything that looks like a parenthesised expression but contains errors
                .recover_with(nested_delimiters(
                    Token::Ctrl('('),
                    Token::Ctrl(')'),
                    [
                        (Token::Ctrl('['), Token::Ctrl(']')),
                        (Token::Ctrl('{'), Token::Ctrl('}')),
                    ],
                    |span| Spanned(Expr::Error, span),
                ))
                // Attempt to recover anything that looks like a list but contains errors
                .recover_with(nested_delimiters(
                    Token::Ctrl('['),
                    Token::Ctrl(']'),
                    [
                        (Token::Ctrl('('), Token::Ctrl(')')),
                        (Token::Ctrl('{'), Token::Ctrl('}')),
                    ],
                    |span| Spanned(Expr::Error, span),
                ));

            // Function calls have very high precedence so we prioritise them
            let call = atom
                .clone()
                .then(
                    items
                        .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
                        .map_with_span(|args, span: Span| (args, span))
                        .repeated(),
                )
                .foldl(|f, args| {
                    let span = f.1.start..args.1.end;
                    Spanned(Expr::Call(Box::new(f), args.0), span)
                });

            let prod_op = just(Token::Op("*".to_string()))
                .to(BinaryOp::Mul)
                .or(just(Token::Op("/".to_string())).to(BinaryOp::Div));

            let neg = just(Token::Op("-".to_string()))
                .repeated()
                .then(atom.clone())
                .foldr(|_op, rhs| {
                    let range = rhs.1.clone();
                    Spanned(Expr::Unary(UnaryOp::Neg, Box::new(rhs)), range)
                });

            let not = just(Token::Not)
                .repeated()
                .then(atom.clone())
                .foldr(|_op, rhs| {
                    let range = rhs.1.clone();
                    Spanned(Expr::Unary(UnaryOp::Not, Box::new(rhs)), range)
                });

            let unary = neg.or(not);

            let product = call
                .clone()
                .or(unary)
                .then(prod_op.then(call).repeated())
                .foldl(|a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    Spanned(Expr::Binary(Box::new(a), op, Box::new(b)), span)
                });

            let op = just(Token::Op("+".to_string()))
                .to(BinaryOp::Add)
                .or(just(Token::Op("-".to_string())).to(BinaryOp::Sub));

            let sum = product
                .clone()
                .then(op.then(product).repeated())
                .foldl(|a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    Spanned(Expr::Binary(Box::new(a), op, Box::new(b)), span)
                });

            let equality_op = just(Token::Op("==".to_string()))
                .to(BinaryOp::Eq)
                .or(just(Token::Op("!=".to_string())).to(BinaryOp::NotEq));

            let compare = sum
                .clone()
                .then(equality_op.then(sum).repeated())
                .foldl(|a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    Spanned(Expr::Binary(Box::new(a), op, Box::new(b)), span)
                });

            let and_op = just(Token::And).to(BinaryOp::And);
            let or_op = just(Token::Or).to(BinaryOp::Or);
            let logical_op = or_op.or(and_op);

            compare
                .clone()
                .then(logical_op.then(compare).repeated())
                .foldl(|a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    Spanned(Expr::Binary(Box::new(a), op, Box::new(b)), span)
                })
        });

        let block_chain = block_expr
            .clone()
            .then(block_expr.clone().repeated())
            .map_with_span(|(a, mut b), span: Span| {
                let e = if b.is_empty() {
                    a
                } else {
                    b.insert(0, a);
                    Spanned(Expr::Sequence(b), span.clone())
                };
                Spanned(Expr::Block(Box::new(e)), span)
            });

        block_chain
            // Expressions, chained by semicolons, are statements
            .or(raw_expr.clone())
            .then(just(Token::Ctrl(';')).ignore_then(expr.or_not()).repeated())
            .map_with_span(|(a, b), span: Span| {
                if b.is_empty() {
                    a
                } else {
                    let mut seq = vec![a];
                    seq.extend(b.into_iter().flatten());
                    Spanned(Expr::Sequence(seq), span.clone())
                }
            })
    })
    .then_ignore(end())
}
