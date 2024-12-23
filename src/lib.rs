use std::io::{self, Write};

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{error::Simple, Parser, Stream};

use crate::{
    ast::{Expr, Span, Spanned},
    bytecode_interpreter::BytecodeInterpreter,
    compiler::{CompileError, Compiler},
    interpreter::Interpreter,
    parser::expr_parser,
};

pub mod ast;
pub mod bytecode_interpreter;
pub mod compiler;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod runtime_value;
pub mod scoped_map;

pub fn run(src: impl AsRef<str>) {
    run_with_interpreter(Interpreter::default(), src);
}

pub fn compile_and_run(src: impl AsRef<str>) {
    let (mut stdout, mut stderr) = (io::stdout(), io::stderr());

    let mut compiler = Compiler::default();

    let compile_res = parse(src.as_ref()).and_then(|ast| {
        compiler.compile(&ast).map_err(|e| {
            vec![match e {
                CompileError::Spanned { span, msg } => chumsky::error::Simple::custom(span, msg),
                CompileError::Plain(msg) => chumsky::error::Simple::custom(Span::default(), msg),
            }]
        })
    });

    let program = match compile_res {
        Ok(program) => program,
        Err(errs) => {
            pretty_print_errors(stdout, src, errs);
            return;
        }
    };

    let res = BytecodeInterpreter::new(program)
        .with_output(&mut stdout, &mut stderr)
        .run()
        .map_err(|(span, err)| vec![Simple::custom(span, err)]);

    if let Err(err) = res {
        pretty_print_errors(stderr, src, err);
    }
}

pub fn run_with_interpreter(
    mut interpreter: Interpreter<impl Write, impl Write>,
    src: impl AsRef<str>,
) {
    let res = parse(src.as_ref()).and_then(|ast| interpreter.run(&ast).map_err(|e| vec![e]));

    if let Err(errs) = res {
        pretty_print_errors(interpreter.stderr, src, errs);
    }
}

pub fn parse(src: impl AsRef<str>) -> Result<Spanned<Expr>, Vec<Simple<String>>> {
    let src = src.as_ref();

    let (tokens, lexer_errs) = lexer::lexer().parse_recovery(src);

    let parse_errs = if let Some(tokens) = tokens {
        let len = src.chars().count();
        let (ast, parse_errs) =
            expr_parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

        dbg!(&ast);

        if let Some(expr) = ast {
            return Ok(expr);
        }

        parse_errs
    } else {
        Vec::new()
    };

    let errs = lexer_errs
        .into_iter()
        .map(|e| e.map(|c| c.to_string()))
        .chain(parse_errs.into_iter().map(|e| e.map(|tok| tok.to_string())));

    Err(errs.collect())
}

pub fn pretty_print_errors(mut sink: impl Write, src: impl AsRef<str>, errs: Vec<Simple<String>>) {
    errs.into_iter().for_each(|e| {
        let report = Report::build(ReportKind::Error, (), e.span().start);

        let report = match e.reason() {
            chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                .with_message(format!(
                    "Unclosed delimiter {}",
                    delimiter.fg(Color::Yellow)
                ))
                .with_label(
                    Label::new(span.clone())
                        .with_message(format!(
                            "Unclosed delimiter {}",
                            delimiter.fg(Color::Yellow)
                        ))
                        .with_color(Color::Yellow),
                )
                .with_label(
                    Label::new(e.span())
                        .with_message(format!(
                            "Must be closed before this {}",
                            e.found()
                                .unwrap_or(&"end of file".to_string())
                                .fg(Color::Red)
                        ))
                        .with_color(Color::Red),
                ),
            chumsky::error::SimpleReason::Unexpected => report
                .with_message(format!(
                    "{}, expected {}",
                    if e.found().is_some() {
                        "Unexpected token in input"
                    } else {
                        "Unexpected end of input"
                    },
                    if e.expected().len() == 0 {
                        "something else".to_string()
                    } else {
                        e.expected()
                            .map(|expected| match expected {
                                Some(expected) => expected.to_string(),
                                None => "end of input".to_string(),
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                ))
                .with_label(
                    Label::new(e.span())
                        .with_message(format!(
                            "Unexpected token {}",
                            e.found()
                                .unwrap_or(&"end of file".to_string())
                                .fg(Color::Red)
                        ))
                        .with_color(Color::Red),
                ),
            chumsky::error::SimpleReason::Custom(msg) => report.with_message(msg).with_label(
                Label::new(e.span())
                    .with_message(format!("{}", msg.fg(Color::Red)))
                    .with_color(Color::Red),
            ),
        };

        report
            .finish()
            .write(Source::from(&src), &mut sink)
            .unwrap();
    });
}
