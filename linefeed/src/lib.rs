#![feature(trait_alias)]

use std::io::{self, Read, Write};

use std::time::Instant;

use ariadne::{Color, Fmt as _, Label, Report, ReportKind, Source};
use chumsky::error::Rich;
use chumsky::prelude::*;
use oxc_allocator::Allocator;

use crate::{
    compiler::Compiler,
    grammar::{
        ast::{Expr, Span, Spanned},
        lexer::{self, Token},
        parser::expr_parser,
    },
    vm::{BytecodeInterpreter, RuntimeError},
};

pub mod compiler;
pub mod grammar;
pub mod vm;

pub use chumsky;

pub fn run(src: impl AsRef<str>) {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    run_with_handles(src, &mut stdin, &mut stdout, &mut stderr);
}

pub fn run_with_handles(
    src: impl AsRef<str>,
    mut stdin: impl Read,
    mut stdout: impl Write,
    mut stderr: impl Write,
) {
    let src = src.as_ref();
    let mut compiler = Compiler::default();

    let parse_start = Instant::now();
    let tokens = match lexer::lexer().parse(src).into_output_errors() {
        (Some(tokens), e) if e.is_empty() => tokens,
        (_, e) => return pretty_print_errors(stderr, src, e),
    };
    let ast = match parse_tokens(src, &tokens) {
        Ok(ast) => ast,
        Err(errs) => return pretty_print_errors(stderr, src, errs),
    };
    let parse_time = Instant::now().duration_since(parse_start);

    let allocator = Allocator::new();

    let compile_start = Instant::now();
    let program = match compiler.compile(&ast, &allocator) {
        Ok(program) => program,
        Err(err) => {
            let span = err.span().unwrap_or(Span::new(0, 0));
            return pretty_print_errors(stderr, src, vec![Rich::<&str>::custom(span, err.msg())]);
        }
    };
    let compile_time = Instant::now().duration_since(compile_start);

    program.disassemble(src.as_ref());

    let run_start = Instant::now();

    let mut bytecode_interpreter = BytecodeInterpreter::new(program, &allocator).with_handles(
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );

    if let Err((span, err)) = bytecode_interpreter.run() {
        return pretty_print_errors(stderr, src, vec![Rich::<RuntimeError>::custom(span, err)]);
    }

    let run_time = Instant::now().duration_since(run_start);
    let instrs_executed = bytecode_interpreter.instructions_executed;

    eprintln!(
        "Parse time: {parse_time:?}, Compile time: {compile_time:?}, Run time: {run_time:?}. {instrs_executed} instructions executed.",
    );
}

pub fn parse_tokens<'src>(
    src: &'src str,
    tokens: &'src [Spanned<Token<'src>>],
) -> Result<Spanned<Expr<'src>>, Vec<Rich<'src, String>>> {
    let (ast, parse_errs) = expr_parser()
        .parse(tokens.map((src.len()..src.len()).into(), |Spanned(t, s)| (t, s)))
        .into_output_errors();

    if !parse_errs.is_empty() {
        return Err(parse_errs
            .into_iter()
            .map(|e| e.map_token(|tok| tok.to_string()))
            .collect());
    }

    Ok(ast.unwrap())
}

pub fn pretty_print_errors(
    mut sink: impl Write,
    src: impl AsRef<str>,
    errs: Vec<Rich<impl ToString + Clone>>,
) {
    let errs = errs.into_iter().map(|e| e.map_token(|c| c.to_string()));

    errs.for_each(|e| {
        let report = Report::build(ReportKind::Error, (), e.span().start);

        let report = match e.reason() {
            chumsky::error::RichReason::ExpectedFound { expected, found } => report
                .with_message(format!(
                    "{}, expected {}",
                    if found.is_some() {
                        "Unexpected token in input"
                    } else {
                        "Unexpected end of input"
                    },
                    if expected.is_empty() {
                        "something else".to_string()
                    } else {
                        expected
                            .iter()
                            .map(|expected| expected.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                ))
                .with_label(
                    Label::new(e.span().into_range())
                        .with_message(format!(
                            "Unexpected token {}",
                            e.found()
                                .unwrap_or(&"end of file".to_string())
                                .fg(Color::Red)
                        ))
                        .with_color(Color::Red),
                ),
            chumsky::error::RichReason::Custom(msg) => report.with_message(msg).with_label(
                Label::new(e.span().into_range())
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
