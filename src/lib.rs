use std::io::{self, Write};

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{error::Simple, Parser, Stream};
use std::time::Instant;

use crate::{
    ast::{Expr, Span, Spanned},
    bytecode_interpreter::BytecodeInterpreter,
    compiler::{CompileError, Compiler},
    interpreter::Interpreter,
    parser::expr_parser,
};

pub mod ast;
pub mod bytecode;
pub mod bytecode_interpreter;
pub mod compiler;
pub mod interpreter;
pub mod ir_value;
pub mod lexer;
pub mod method;
pub mod parser;
pub mod runtime_value;
pub mod scoped_map;

pub fn run(src: impl AsRef<str>) {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    run_with_output(src, &mut stdout, &mut stderr);
}

pub fn run_with_output(src: impl AsRef<str>, mut stdout: impl Write, mut stderr: impl Write) {
    let mut compiler = Compiler::default();

    let parse_start = Instant::now();
    let ast = parse(src.as_ref());
    let parse_time = Instant::now().duration_since(parse_start);

    let compile_start = Instant::now();
    let compile_res = ast.and_then(|ast| {
        compiler.compile(&ast).map_err(|e| {
            vec![match e {
                CompileError::Spanned { span, msg } => chumsky::error::Simple::custom(span, msg),
                CompileError::Plain(msg) => chumsky::error::Simple::custom(Span::default(), msg),
            }]
        })
    });
    let compile_time = Instant::now().duration_since(compile_start);

    let program = match compile_res {
        Ok(program) => program,
        Err(errs) => {
            pretty_print_errors(stderr, src, errs);
            return;
        }
    };

    program.disassemble(src.as_ref());

    let run_start = Instant::now();
    let res = BytecodeInterpreter::new(program)
        .with_output(&mut stdout, &mut stderr)
        .run()
        .map_err(|(span, err)| vec![Simple::custom(span, err)]);
    let run_time = Instant::now().duration_since(run_start);

    if let Err(err) = res {
        pretty_print_errors(stderr, src, err);
    }

    eprintln!(
        "Parse time: {:?}, Compile time: {:?}, Run time: {:?}",
        parse_time, compile_time, run_time
    );
}

pub fn run_with_interpreter(
    mut interpreter: Interpreter<impl Write, impl Write>,
    src: impl AsRef<str>,
) {
    let parse_start = Instant::now();
    let ast = parse(src.as_ref());
    let parse_time = Instant::now().duration_since(parse_start);

    let run_start = Instant::now();
    let res = ast.and_then(|ast| interpreter.run(&ast).map_err(|e| vec![e]));
    let run_time = Instant::now().duration_since(run_start);

    if let Err(errs) = res {
        pretty_print_errors(interpreter.stderr, src, errs);
    }

    eprintln!("Parse time: {:?}, Run time: {:?}", parse_time, run_time);
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
