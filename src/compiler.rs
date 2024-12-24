// TODO: Make all arguments generic/polymorphic, generate code for all possible types. Type inference.

use std::{cell::RefCell, iter, rc::Rc};

use crate::{
    ast::{Expr, Span, Spanned, UnaryOp, Value as AstValue},
    runtime_value::RuntimeValue,
    scoped_map::ScopedMap,
};

#[derive(Debug, Clone)]
pub enum Instruction {
    Load,
    Store,
    PrintValue,
    Value(RuntimeValue),
    GetBasePtr,
    Add,
    Sub,
    Mul,
    Div,
    ConstantInt(isize),
    Not,
    Stop,
    Goto(usize),
    Append,
}

use Instruction::*;

#[derive(Debug, Default)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub source_map: Vec<Span>,
}

#[derive(Default)]
pub struct Compiler {
    vars: ScopedMap<String, usize>,
}

impl Compiler {
    pub fn compile(&mut self, expr: &Spanned<Expr>) -> Result<Program, CompileError> {
        let program = self
            .compile_expr(expr)?
            .then_instructions(vec![Stop], expr.1.end..expr.1.end);

        assert_eq!(program.instructions.len(), program.source_map.len());

        Ok(program)
    }

    fn compile_expr(&mut self, expr: &Spanned<Expr>) -> Result<Program, CompileError> {
        let instructions = match &expr.0 {
            Expr::Error => unreachable!(),

            Expr::Local(name) => {
                let instrs =
                    self.compile_var_access(name)
                        .map_err(|msg| CompileError::Spanned {
                            span: expr.1.clone(),
                            msg: format!("Failed to compile variable access to '{name}': {msg}"),
                        })?;

                Program::from_instructions(instrs, expr.1.clone())
            }

            Expr::Let(name, val) => self.compile_var_assign(expr, name, val)?,

            Expr::Value(val) => {
                let instrs = self
                    .compile_value(val)
                    .map_err(|msg| CompileError::Spanned {
                        span: expr.1.clone(),
                        msg,
                    })?;

                Program::from_instructions(instrs, expr.1.clone())
            }

            Expr::Sequence(exprs) => exprs
                .iter()
                .map(|expr| self.compile_expr(expr))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .fold(Program::default(), Program::then_program),

            Expr::Print(sub_expr) => self
                .compile_expr(sub_expr)?
                .then_instructions(vec![PrintValue], expr.1.clone()),

            Expr::Block(sub_expr) => self.compile_expr(sub_expr)?,

            Expr::Unary(op, sub_expr) => {
                let program = self.compile_expr(sub_expr)?;

                let to_add = match op {
                    UnaryOp::Not => vec![Not],
                    UnaryOp::Neg => vec![ConstantInt(-1), Mul],
                };

                program.then_instructions(to_add, expr.1.clone())
            }

            Expr::List(items) => {
                let initial_val = Program::default().then_instructions(
                    vec![Value(RuntimeValue::List(Rc::new(RefCell::new(vec![]))))],
                    expr.1.clone(),
                );

                items
                    .iter()
                    .map(|item| self.compile_expr(item))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .fold(initial_val, |acc, p| {
                        acc.then_program(p)
                            .then_instructions(vec![Append], expr.1.clone())
                    })
            }

            _ => unimplemented!(),
        };

        Ok(instructions)
    }

    // FIXME: The addresses here are completely nonsensical for outer scopes due to
    // base-pointer-relative addressing
    fn compile_var_access(&mut self, name: &String) -> Result<Vec<Instruction>, String> {
        let addr = self
            .vars
            .get(name)
            .ok_or_else(|| format!("Variable {name} not found"))?;

        Ok(vec![GetBasePtr, ConstantInt(*addr as isize), Add, Load])
    }

    // FIXME: Same as above
    fn compile_var_assign(
        &mut self,
        expr: &Spanned<Expr>,
        name: &String,
        val: &Spanned<Expr>,
    ) -> Result<Program, CompileError> {
        let addr = self.vars.get(name).cloned().unwrap_or_else(|| {
            let local_addr = self.vars.cur_scope_len();
            self.vars.set(name.clone(), local_addr);
            local_addr
        });

        let program = Program::default()
            .then_instructions(
                vec![GetBasePtr, ConstantInt(addr as isize), Add],
                expr.1.clone(),
            )
            .then_program(self.compile_expr(val)?)
            .then_instructions(vec![Store], expr.1.clone());

        Ok(program)
    }

    fn compile_value(&mut self, val: &AstValue) -> Result<Vec<Instruction>, String> {
        let rt_val = RuntimeValue::try_from(val)?;
        Ok(vec![Instruction::Value(rt_val)])
    }
}

fn repeat_span(span: Span, count: usize) -> Vec<Span> {
    iter::repeat(span).take(count).collect()
}

impl Program {
    pub fn disassemble(&self, src: &str) {
        for (instr, span) in self.instructions.iter().zip(&self.source_map) {
            let i = format!("{:?}", instr);
            let range = format!("{:?}", span);
            println!("{:>20}  {:<8} {:?}", i, range, &src[span.start..span.end]);
        }
    }

    pub fn from_instructions(instrs: Vec<Instruction>, span: Span) -> Program {
        Program {
            source_map: repeat_span(span, instrs.len()),
            instructions: instrs,
        }
    }

    pub fn add_instructions(&mut self, instrs: Vec<Instruction>, span: Span) {
        self.source_map.extend(repeat_span(span, instrs.len()));
        self.instructions.extend(instrs);
    }

    pub fn then_instructions(mut self, instrs: Vec<Instruction>, span: Span) -> Program {
        self.add_instructions(instrs, span);
        self
    }

    pub fn extend(&mut self, other: Program) {
        assert_eq!(self.instructions.len(), self.source_map.len());
        self.instructions.extend(other.instructions);
        self.source_map.extend(other.source_map);
    }

    pub fn then_program(mut self, other: Program) -> Program {
        self.extend(other);
        self
    }
}

pub enum CompileError {
    Spanned { span: Span, msg: String },
    Plain(String),
}

impl TryFrom<&AstValue> for RuntimeValue {
    type Error = String;

    fn try_from(val: &AstValue) -> Result<Self, Self::Error> {
        let res = match val {
            AstValue::Null => RuntimeValue::Null,
            AstValue::Bool(b) => RuntimeValue::Bool(*b),
            AstValue::Num(n) => RuntimeValue::Num(*n),
            AstValue::Str(s) => RuntimeValue::Str(Rc::new(s.clone())),
            AstValue::List(xs) => {
                let items = xs
                    .iter()
                    .map(RuntimeValue::try_from)
                    .collect::<Result<_, _>>()?;

                RuntimeValue::List(Rc::new(RefCell::new(items)))
            }
            AstValue::Func(_) => return Err("Cannot compile function value".to_string()),
        };

        Ok(res)
    }
}
