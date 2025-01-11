use std::rc::Rc;

use chumsky::{input::ValueInput, prelude::*};

use crate::ast::{AstValue, BinaryOp, Expr, Func, Span, Spanned, UnaryOp};
use crate::lexer::Token;

pub fn expr_parser<'src, I>(
) -> impl Parser<'src, I, Spanned<Expr<'src>>, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
{
    recursive(|expr| {
        let nested_braces_delim = nested_delimiters(
            Token::Ctrl('{'),
            Token::Ctrl('}'),
            [
                (Token::Ctrl('('), Token::Ctrl(')')),
                (Token::Ctrl('['), Token::Ctrl(']')),
            ],
            |span| Spanned(Expr::ParseError, span),
        );

        let block = expr
            .clone()
            .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}')))
            .map_with(|expr, e| Spanned(Expr::Block(Box::new(expr)), e.span()))
            .recover_with(via_parser(nested_braces_delim.clone()))
            .memoized()
            .boxed();

        let if_ = recursive(|if_| {
            just(Token::If)
                .ignore_then(expr.clone())
                .then(block.clone())
                .then(
                    just(Token::Else)
                        .ignore_then(block.clone().or(if_))
                        .or_not(),
                )
                .map_with(|((cond, a), b), e| {
                    let span: Span = e.span();
                    Spanned(
                        Expr::If(
                            Box::new(cond),
                            Box::new(a),
                            Box::new(match b {
                                Some(b) => b,
                                // If an `if` expression has no trailing `else` block, we magic up one that just produces null
                                None => Spanned(Expr::Value(AstValue::Null), span.to_end()),
                            }),
                        ),
                        e.span(),
                    )
                })
                .memoized()
        });

        let while_ = just(Token::While)
            .ignore_then(expr.clone())
            .then(block.clone())
            .map_with(|(cond, a), e| Spanned(Expr::While(Box::new(cond), Box::new(a)), e.span()))
            .memoized();

        let ident = select! { Token::Ident(ident) => ident }.labelled("identifier");

        let for_ = just(Token::For)
            .ignore_then(ident)
            .then(just(Token::In).ignore_then(expr.clone()))
            .then(block.clone())
            .map_with(|((var, iter), body), e| {
                Spanned(Expr::For(var, Box::new(iter), Box::new(body)), e.span())
            })
            .memoized();

        let block_expr = choice((block.clone(), if_, while_, for_))
            .memoized()
            .boxed()
            .labelled("block expression");

        let raw_expr = recursive(|raw_expr| {
            let val = select! {
                Token::Null => Expr::Value(AstValue::Null),
                Token::Bool(x) => Expr::Value(AstValue::Bool(x)),
                Token::Num(n) => Expr::Value(AstValue::Num(n)),
                Token::Str(s) => Expr::Value(AstValue::Str(s)),
                Token::Regex(r) => Expr::Value(AstValue::Regex(r)),
                // TODO: for cleanliness, this should probably not be a "value" - make a separate
                // keyword parser?
                Token::Break => Expr::Break,
                Token::Continue => Expr::Continue,
            }
            .labelled("value");

            // A comma-separated list of expressions
            let items = expr
                .clone()
                .separated_by(just(Token::Ctrl(',')))
                .allow_trailing()
                .collect::<Vec<_>>()
                .boxed();

            // Argument lists are just identifiers separated by commas, surrounded by parentheses
            let args = ident
                .separated_by(just(Token::Ctrl(',')))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
                .labelled("function args")
                .memoized()
                .boxed();

            let func = just(Token::Fn)
                .ignore_then(ident.or_not().labelled("function name"))
                .then(args)
                .then(
                    block_expr
                        .clone()
                        // Attempt to recover anything that looks like a function body but contains errors
                        .recover_with(via_parser(nested_braces_delim.clone()))
                        .or(raw_expr.clone()),
                )
                .map_with(|((name, args), body), e| {
                    let val = Expr::Value(AstValue::Func(Func {
                        args,
                        body: Rc::new(body),
                    }));

                    match name {
                        Some(name) => Expr::Let(name, Box::new(Spanned(val, e.span()))),
                        None => val,
                    }
                })
                .labelled("function")
                .memoized()
                .boxed();

            let destructure_assign = ident
                .separated_by(just(Token::Ctrl(',')))
                .at_least(2)
                .collect::<Vec<_>>()
                .then_ignore(just(Token::Op("=")))
                .then(raw_expr.clone().or(block_expr.clone()))
                .map(|(vars, val)| Expr::Destructure(vars, Box::new(val)))
                .memoized()
                .boxed();

            // TODO: This should probably be in the lexer
            // Variable assignment
            let assign_op = choice((
                just(Token::Op("=")).to("="),
                just(Token::Op("+=")).to("+="),
                just(Token::Op("-=")).to("-="),
                just(Token::Op("*=")).to("*="),
                just(Token::Op("/=")).to("/="),
                just(Token::Op("%=")).to("%="),
            ));

            let single_assign = ident
                .then(assign_op)
                .then(raw_expr.clone().or(block_expr.clone()))
                .map_with(|((name, op), val), e| {
                    let new_val = match op {
                        "=" => val,
                        _ => Spanned(
                            Expr::Binary(
                                Box::new(Spanned(Expr::Local(name), e.span())),
                                match op {
                                    "+=" => BinaryOp::Add,
                                    "-=" => BinaryOp::Sub,
                                    "*=" => BinaryOp::Mul,
                                    "/=" => BinaryOp::Div,
                                    "%=" => BinaryOp::Mod,
                                    _ => unreachable!(),
                                },
                                Box::new(val),
                            ),
                            e.span(),
                        ),
                    };

                    Expr::Let(name, Box::new(new_val))
                })
                .memoized()
                .boxed();

            let let_ = choice((destructure_assign, single_assign)).labelled("assignment");

            let list = items
                .clone()
                .delimited_by(just(Token::Ctrl('[')), just(Token::Ctrl(']')))
                .map(Expr::List);

            let list_comprehension = expr
                .clone()
                .then(just(Token::For).ignore_then(ident))
                .then(just(Token::In).ignore_then(expr.clone()))
                .delimited_by(just(Token::Ctrl('[')), just(Token::Ctrl(']')))
                .map(|((body, loop_var), iter)| {
                    Expr::ListComprehension(Box::new(body), loop_var, Box::new(iter))
                })
                .memoized()
                .boxed();

            let tuple = expr
                .clone()
                .separated_by(just(Token::Ctrl(',')))
                .at_least(2)
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
                .map(Expr::Tuple)
                .memoized();

            // 'Atoms' are expressions that contain no ambiguity
            let atom = val
                .or(let_)
                .or(list)
                .or(tuple)
                .or(list_comprehension)
                .or(func)
                .or(ident.map(Expr::Local))
                .map_with(|expr, e| Spanned(expr, e.span()))
                // Atoms can also just be normal expressions, but surrounded with parentheses
                .or(expr
                    .clone()
                    .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')'))))
                // Attempt to recover anything that looks like a parenthesised expression but contains errors
                .recover_with(via_parser(nested_braces_delim))
                .memoized()
                .boxed(); // Boxing significantly improves compile time

            let call_with_args = items
                .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
                .map_with(|args, e| Spanned(args, e.span()))
                .labelled("function call args")
                .memoized()
                .boxed();

            // Function calls have very high precedence so we prioritise them
            let func_call = atom
                .clone()
                .foldl_with(
                    call_with_args.clone().repeated().at_least(1),
                    |f, args, e| Spanned(Expr::Call(Box::new(f), args.0), e.span()),
                )
                .labelled("function call")
                .memoized()
                .boxed();

            let index = expr
                .clone()
                .delimited_by(just(Token::Ctrl('[')), just(Token::Ctrl(']')));
            let index_into = atom
                .clone()
                .foldl_with(index.repeated().at_least(1), |val, idx, e| {
                    Spanned(Expr::Index(Box::new(val), Box::new(idx)), e.span())
                })
                .memoized()
                .boxed();

            let call_or_index = choice((func_call, index_into, atom.clone()));

            let method_call = call_or_index
                .clone()
                .foldl_with(
                    just(Token::Ctrl('.'))
                        .ignore_then(ident)
                        .then(call_with_args)
                        .repeated()
                        .at_least(1),
                    |val, (method, args), e| {
                        Spanned(Expr::MethodCall(Box::new(val), method, args.0), e.span())
                    },
                )
                .memoized()
                .boxed();

            let with_method_call = choice((method_call, call_or_index)).boxed();

            let neg = just(Token::Op("-"))
                .repeated()
                .foldr_with(atom.clone(), |_op, rhs, e| {
                    Spanned(Expr::Unary(UnaryOp::Neg, Box::new(rhs)), e.span())
                });

            let not = just(Token::Not)
                .repeated()
                .foldr_with(atom.clone(), |_op, rhs, e| {
                    Spanned(Expr::Unary(UnaryOp::Not, Box::new(rhs)), e.span())
                });

            let unary = neg.or(not).memoized().boxed();

            let prod_op = choice((
                just(Token::Op("*")).to(BinaryOp::Mul),
                just(Token::Op("/")).to(BinaryOp::Div),
                just(Token::Op("%")).to(BinaryOp::Mod),
            ));

            let product = with_method_call
                .clone()
                .or(unary)
                .foldl_with(
                    prod_op.then(with_method_call).repeated(),
                    |a, (op, b), e| Spanned(Expr::Binary(Box::new(a), op, Box::new(b)), e.span()),
                )
                .memoized()
                .boxed();

            let sum_op = choice((
                just(Token::Op("+")).to(BinaryOp::Add),
                just(Token::Op("-")).to(BinaryOp::Sub),
            ));

            let sum = product
                .clone()
                .foldl_with(sum_op.then(product).repeated(), |a, (op, b), e| {
                    Spanned(Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
                })
                .memoized()
                .boxed();

            let cmp_op = choice((
                just(Token::Op("==")).to(BinaryOp::Eq),
                just(Token::Op("!=")).to(BinaryOp::NotEq),
                just(Token::Op("<")).to(BinaryOp::Less),
                just(Token::Op("<=")).to(BinaryOp::LessEq),
                just(Token::Op(">")).to(BinaryOp::Greater),
                just(Token::Op(">=")).to(BinaryOp::GreaterEq),
            ));

            let compare = sum
                .clone()
                .foldl_with(cmp_op.then(sum).repeated(), |a, (op, b), e| {
                    Spanned(Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
                })
                .memoized()
                .boxed();

            let logical_op = choice((
                just(Token::And).to(BinaryOp::And),
                just(Token::Or).to(BinaryOp::Or),
                just(Token::Xor).to(BinaryOp::Xor),
            ));

            let logical = compare
                .clone()
                .foldl_with(logical_op.then(compare).repeated(), |a, (op, b), e| {
                    Spanned(Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
                })
                .memoized()
                .boxed();

            let range_op = just(Token::Op("..")).to(BinaryOp::Range);
            let range = logical
                .clone()
                .then(range_op.then(logical.clone().or_not()))
                .map_with(|(a, (op, b)), e| {
                    let end = b.unwrap_or_else(|| Spanned(Expr::Value(AstValue::Null), e.span()));
                    Spanned(Expr::Binary(Box::new(a), op, Box::new(end)), e.span())
                })
                .labelled("range")
                .memoized()
                .boxed();

            let return_ = just(Token::Return)
                .ignore_then(raw_expr.clone().or(block_expr.clone()).or_not())
                .map_with(|expr, e| {
                    let ret_expr =
                        expr.unwrap_or_else(|| Spanned(Expr::Value(AstValue::Null), e.span()));
                    Spanned(Expr::Return(Box::new(ret_expr)), e.span())
                })
                .labelled("return")
                .memoized()
                .boxed();

            choice((range, logical, return_))
        });

        let postfix_if = raw_expr
            .clone()
            .then(just(Token::If).ignore_then(raw_expr.clone()))
            .map_with(|(a, b), e| {
                Spanned(
                    Expr::If(
                        Box::new(b),
                        Box::new(a),
                        Box::new(Spanned(Expr::Value(AstValue::Null), e.span())),
                    ),
                    e.span(),
                )
            })
            .memoized()
            .boxed();

        let postfix_unless = raw_expr
            .clone()
            .then(just(Token::Unless).ignore_then(raw_expr.clone()))
            .map_with(|(a, b), e| {
                Spanned(
                    Expr::If(
                        Box::new(Spanned(Expr::Unary(UnaryOp::Not, Box::new(b)), e.span())),
                        Box::new(a),
                        Box::new(Spanned(Expr::Value(AstValue::Null), e.span())),
                    ),
                    e.span(),
                )
            })
            .memoized()
            .boxed();

        // TODO: What does this parser even do anymore? Discard it and keep only the below?
        let block_chain = block_expr
            .clone()
            .then(block_expr.repeated().collect::<Vec<_>>())
            .map_with(|(a, mut b), e| {
                let span: Span = e.span();
                let e = if b.is_empty() {
                    a
                } else {
                    b.insert(0, a);
                    Spanned(Expr::Sequence(b), span)
                };
                Spanned(Expr::Block(Box::new(e)), span)
            })
            .memoized()
            .boxed();

        block_chain
            .or(postfix_if)
            .or(postfix_unless)
            // Expressions, chained by semicolons, are statements
            .or(raw_expr.clone())
            .then(
                just(Token::Ctrl(';'))
                    .ignore_then(expr.or_not())
                    .repeated()
                    .collect::<Vec<_>>()
                    .memoized(),
            )
            .map_with(|(a, b), e| {
                if b.is_empty() {
                    a
                } else {
                    let mut seq = vec![a];
                    seq.extend(b.into_iter().flatten());
                    Spanned(Expr::Sequence(seq), e.span())
                }
            })
            .memoized()
            .boxed()
    })
    .then_ignore(end())
}
