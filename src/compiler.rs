// TODO: Make all arguments generic/polymorphic, generate code for all possible types. Type inference.

use std::{iter, rc::Rc};

use crate::{
    ast::{Expr, Span, Spanned, UnaryOp, Value as AstValue},
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
}

use Instruction::*;

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Null,
    Bool(bool),
    Int(isize),
    Num(f64),
    Str(Rc<String>),
    List(Rc<Vec<RuntimeValue>>),
}

const _: () = {
    // Just to make sure that we don't accidentally change the size of RuntimeValue and make
    // cloning super expensive
    assert!(std::mem::size_of::<RuntimeValue>() == 16);
};

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
        let mut program = self.compile_expr(expr)?;

        program.instructions.push(Stop);
        program.source_map.push(expr.1.end..expr.1.end);

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

                Program {
                    source_map: repeat_span(expr.1.clone(), instrs.len()),
                    instructions: instrs,
                }
            }

            Expr::Let(name, val) => self.compile_var_assign(expr, name, val)?,

            Expr::Value(val) => {
                let instrs = self
                    .compile_value(val)
                    .map_err(|msg| CompileError::Spanned {
                        span: expr.1.clone(),
                        msg,
                    })?;

                Program {
                    source_map: repeat_span(expr.1.clone(), instrs.len()),
                    instructions: instrs,
                }
            }

            Expr::Sequence(exprs) => exprs
                .iter()
                .map(|expr| self.compile_expr(expr))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .fold(Program::default(), |mut program, sub_program| {
                    program.instructions.extend(sub_program.instructions);
                    program.source_map.extend(sub_program.source_map);
                    program
                }),

            Expr::Print(expr) => {
                let mut instrs = self.compile_expr(expr)?;
                instrs.instructions.push(PrintValue);
                instrs.source_map.push(expr.1.clone());
                instrs
            }

            Expr::Block(expr) => self.compile_expr(expr)?,

            Expr::Unary(op, expr) => {
                let mut instrs = self.compile_expr(expr)?;

                let to_add = match op {
                    UnaryOp::Not => vec![Not],
                    UnaryOp::Neg => vec![ConstantInt(-1), Mul],
                };

                instrs
                    .source_map
                    .extend(repeat_span(expr.1.clone(), to_add.len()));
                instrs.instructions.extend(to_add);

                instrs
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

        let mut program = Program::default();

        let store_instrs = vec![GetBasePtr, ConstantInt(addr as isize), Add];
        program
            .source_map
            .extend(repeat_span(expr.1.clone(), store_instrs.len()));
        program.instructions.extend(store_instrs);

        let val_program = self.compile_expr(val)?;
        program.instructions.extend(val_program.instructions);
        program.source_map.extend(val_program.source_map);

        program.instructions.push(Store);
        program.source_map.push(expr.1.clone());

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

pub enum CompileError {
    Spanned { span: Span, msg: String },
    Plain(String),
}

impl RuntimeValue {
    pub fn kind_str(&self) -> &str {
        match self {
            RuntimeValue::Null => "null",
            RuntimeValue::Bool(_) => "boolean",
            RuntimeValue::Int(_) => "integer",
            RuntimeValue::Num(_) => "number",
            RuntimeValue::Str(_) => "str",
            RuntimeValue::List(_) => "list",
        }
    }
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

                RuntimeValue::List(Rc::new(items))
            }
            AstValue::Func(_) => return Err("Cannot compile function value".to_string()),
        };

        Ok(res)
    }
}
