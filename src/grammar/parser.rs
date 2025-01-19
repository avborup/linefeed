use std::rc::Rc;

use chumsky::{input::ValueInput, prelude::*};

use crate::lexer::Token;
use crate::{
    grammar::ast::{AstValue, BinaryOp, Expr, Func, Span, Spanned, UnaryOp},
    vm::runtime_value::regex::RegexModifiers,
};

pub trait Parser<'src, I, T> =
    chumsky::Parser<'src, I, T, extra::Err<Rich<'src, Token<'src>, Span>>> + Clone + 'src
    where I: ValueInput<'src, Token = Token<'src>, Span = Span>;

pub trait ParserInput<'src> = ValueInput<'src, Token = Token<'src>, Span = Span>;

type BoxedParser<'src, 'b, I> =
    Boxed<'src, 'b, I, Spanned<Expr<'src>>, extra::Err<Rich<'src, Token<'src>, Span>>>;

pub fn expr_parser<'src, I: ParserInput<'src>>() -> impl Parser<'src, I, Spanned<Expr<'src>>> {
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

        let ident = ident_parser();

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

        let inline_expr = recursive(|inline_expr| {
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
                        .or(inline_expr.clone()),
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

            let match_arms = inline_expr
                .clone()
                .then_ignore(just(Token::Op("=>")))
                .then(expr.clone())
                .separated_by(just(Token::Ctrl(',')))
                .allow_trailing()
                .collect::<Vec<_>>();
            let match_expr = just(Token::Match)
                .ignore_then(inline_expr.clone())
                .then(match_arms.delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}'))))
                .map(|(expr, arms)| Expr::Match(Box::new(expr), arms));

            let destructure_assign = ident
                .separated_by(just(Token::Ctrl(',')))
                .at_least(2)
                .collect::<Vec<_>>()
                .then_ignore(just(Token::Op("=")))
                .then(inline_expr.clone())
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
                .then(inline_expr.clone())
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

            let map = map_parser(inline_expr.clone());

            let regex_modifiers = ident
                .or_not()
                .map(|ident| {
                    ident
                        .map(|i| i.chars().collect::<Vec<_>>())
                        .unwrap_or_default()
                })
                .map(|mods| RegexModifiers {
                    case_insensitive: mods.contains(&'i'),
                    parse_nums: mods.contains(&'n'),
                });

            let regex = select! { Token::Regex(r) => r }
                .then(regex_modifiers)
                .map(|(r, m)| Expr::Value(AstValue::Regex(r, m)))
                .memoized()
                .labelled("regex");

            let val = value_parser();
            let standalone_keyword = standalone_keyword_parser();

            // 'Atoms' are expressions that contain no ambiguity
            let atom = val
                .or(standalone_keyword)
                .or(regex)
                .or(let_)
                .or(list)
                .or(tuple)
                .or(map)
                .or(list_comprehension)
                .or(func)
                .or(match_expr)
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

            let index_into = index_into_parser(atom.clone(), inline_expr.clone());

            let index_assign = index_assign_parser(index_into.clone(), inline_expr.clone())
                .labelled("index assignment");

            let call_or_index = func_call.or(index_assign).or(index_into).or(atom.clone());

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

            // The order in the chain corresponds to operator precedence: earlier in the chain,
            // higher precedence
            let logical = chain_parsers(
                with_method_call.or(unary).boxed(),
                [
                    product_parser,
                    sum_parser,
                    compare_parser,
                    contains_parser,
                    logical_parser,
                ],
            );

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
                .ignore_then(inline_expr.clone().or_not())
                .map_with(|expr, e| {
                    let ret_expr =
                        expr.unwrap_or_else(|| Spanned(Expr::Value(AstValue::Null), e.span()));
                    Spanned(Expr::Return(Box::new(ret_expr)), e.span())
                })
                .labelled("return")
                .memoized()
                .boxed();

            range.or(logical).or(block_expr.clone()).or(return_)
        });

        let postfix_if = inline_expr
            .clone()
            .then(just(Token::If).ignore_then(inline_expr.clone()))
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

        let postfix_unless = inline_expr
            .clone()
            .then(just(Token::Unless).ignore_then(inline_expr.clone()))
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
            .or(inline_expr.clone())
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

fn value_parser<'src, I: ParserInput<'src>>() -> impl Parser<'src, I, Expr<'src>> + Copy {
    select! {
        Token::Null => Expr::Value(AstValue::Null),
        Token::Bool(x) => Expr::Value(AstValue::Bool(x)),
        Token::Num(n) => Expr::Value(AstValue::Num(n)),
        Token::Str(s) => Expr::Value(AstValue::Str(s)),
    }
    .labelled("value")
}

fn standalone_keyword_parser<'src, I: ParserInput<'src>>() -> impl Parser<'src, I, Expr<'src>> + Copy
{
    select! {
        Token::Break => Expr::Break,
        Token::Continue => Expr::Continue,
    }
    .labelled("standalone keyword")
}

fn ident_parser<'src, I: ParserInput<'src>>() -> impl Parser<'src, I, &'src str> + Copy {
    select! { Token::Ident(ident) => ident }.labelled("identifier")
}

fn map_parser<'src, I: ParserInput<'src>>(
    inline_expr: impl Parser<'src, I, Spanned<Expr<'src>>>,
) -> impl Parser<'src, I, Expr<'src>> {
    let key_val = inline_expr
        .clone()
        .then_ignore(just(Token::Ctrl(':')))
        .then(inline_expr);

    let map = key_val
        .separated_by(just(Token::Ctrl(',')))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}')))
        .map(Expr::Map)
        .memoized();

    map.labelled("map")
}

fn product_parser<'src, I: ParserInput<'src>>(
    prev: impl Parser<'src, I, Spanned<Expr<'src>>>,
) -> BoxedParser<'src, 'src, I> {
    let prod_op = choice((
        just(Token::Op("*")).to(BinaryOp::Mul),
        just(Token::Op("/")).to(BinaryOp::Div),
        just(Token::Op("%")).to(BinaryOp::Mod),
    ));

    prev.clone()
        .foldl_with(prod_op.then(prev).repeated(), |a, (op, b), e| {
            Spanned(Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
        })
        .memoized()
        .boxed()
}

fn sum_parser<'src, I: ParserInput<'src>>(
    prev: impl Parser<'src, I, Spanned<Expr<'src>>>,
) -> BoxedParser<'src, 'src, I> {
    let sum_op = choice((
        just(Token::Op("+")).to(BinaryOp::Add),
        just(Token::Op("-")).to(BinaryOp::Sub),
    ));

    prev.clone()
        .foldl_with(sum_op.then(prev).repeated(), |a, (op, b), e| {
            Spanned(Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
        })
        .memoized()
        .boxed()
}

fn compare_parser<'src, I: ParserInput<'src>>(
    prev: impl Parser<'src, I, Spanned<Expr<'src>>>,
) -> BoxedParser<'src, 'src, I> {
    let cmp_op = choice((
        just(Token::Op("==")).to(BinaryOp::Eq),
        just(Token::Op("!=")).to(BinaryOp::NotEq),
        just(Token::Op("<")).to(BinaryOp::Less),
        just(Token::Op("<=")).to(BinaryOp::LessEq),
        just(Token::Op(">")).to(BinaryOp::Greater),
        just(Token::Op(">=")).to(BinaryOp::GreaterEq),
    ));

    prev.clone()
        .foldl_with(cmp_op.then(prev).repeated(), |a, (op, b), e| {
            Spanned(Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
        })
        .memoized()
        .boxed()
}

fn contains_parser<'src, I>(
    prev: impl Parser<'src, I, Spanned<Expr<'src>>>,
) -> BoxedParser<'src, 'src, I> {
    prev.clone()
        .foldl_with(
            just(Token::Not)
                .or_not()
                .then_ignore(just(Token::In))
                .then(prev)
                .repeated(),
            |a, (not, b), e| {
                let is_in = Expr::Binary(Box::new(a), BinaryOp::In, Box::new(b));

                let check = if not.is_some() {
                    Expr::Unary(UnaryOp::Not, Box::new(Spanned(is_in, e.span())))
                } else {
                    is_in
                };

                Spanned(check, e.span())
            },
        )
        .memoized()
        .boxed()
}

fn logical_parser<'src, I: ParserInput<'src>>(
    prev: impl Parser<'src, I, Spanned<Expr<'src>>>,
) -> BoxedParser<'src, 'src, I> {
    let logical_op = choice((
        just(Token::And).to(BinaryOp::And),
        just(Token::Or).to(BinaryOp::Or),
        just(Token::Xor).to(BinaryOp::Xor),
    ));

    prev.clone()
        .foldl_with(logical_op.then(prev).repeated(), |a, (op, b), e| {
            Spanned(Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
        })
        .memoized()
        .boxed()
}

fn chain_parsers<'src, 'b, I, F>(
    prev: BoxedParser<'src, 'b, I>,
    parsers: impl IntoIterator<Item = F>,
) -> BoxedParser<'src, 'b, I>
where
    I: ValueInput<'src, Token = Token<'src>, Span = Span>,
    F: FnOnce(BoxedParser<'src, 'b, I>) -> BoxedParser<'src, 'b, I>,
    'src: 'b,
    'b: 'src,
{
    parsers
        .into_iter()
        .fold(prev, move |prev, parser| parser(prev))
}

fn index_into_parser<'src, I: ParserInput<'src>>(
    atom: impl Parser<'src, I, Spanned<Expr<'src>>>,
    expr: impl Parser<'src, I, Spanned<Expr<'src>>>,
) -> impl Parser<'src, I, Spanned<Expr<'src>>> {
    let index = expr.delimited_by(just(Token::Ctrl('[')), just(Token::Ctrl(']')));

    let index_into = atom.foldl_with(index.repeated().at_least(1), |val, idx, e| {
        Spanned(Expr::Index(Box::new(val), Box::new(idx)), e.span())
    });

    index_into.labelled("index").memoized()
}

fn index_assign_parser<'src, I: ParserInput<'src>>(
    index_parser: impl Parser<'src, I, Spanned<Expr<'src>>>,
    val_parser: impl Parser<'src, I, Spanned<Expr<'src>>>,
) -> impl Parser<'src, I, Spanned<Expr<'src>>> {
    index_parser
        .then_ignore(just(Token::Op("=")))
        .then(val_parser)
        .map_with(|(indexed, value), e| match indexed.0 {
            Expr::Index(indexee, idx) => {
                Spanned(Expr::IndexAssign(indexee, idx, Box::new(value)), e.span())
            }
            _ => unreachable!(),
        })
}
