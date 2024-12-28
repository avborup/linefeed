// TODO: Make all arguments generic/polymorphic, generate code for all possible types. Type inference.

use std::iter;

use crate::{
    ast::{BinaryOp, Expr, Span, Spanned, UnaryOp, Value as AstValue},
    bytecode::Bytecode,
    ir_value::{IrList, IrValue},
    runtime_value::{function::RuntimeFunction, number::RuntimeNumber},
    scoped_map::{ScopedMap, VarType},
};

#[derive(Debug, Clone)]
pub enum Instruction {
    // Variables
    Load,
    Store,

    // Values
    Value(IrValue),
    ConstantInt(isize),

    // Stack manipulation
    Pop,

    // Binary operations
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Less,
    LessEq,
    Greater,
    GreaterEq,

    // Unary operations
    Not,

    // Control flow
    Stop,
    Goto(Label),
    Label(Label),
    IfTrue(Label),
    IfFalse(Label),

    // Functions
    GetBasePtr,
    Call(usize),
    Return,

    // Methods
    Method(Method),

    // Builtins
    PrintValue,
}

#[derive(Debug, Clone)]
pub enum Method {
    Append,
}

use Instruction::*;

#[derive(Debug, Default)]
pub struct Program<T> {
    pub instructions: Vec<T>,
    pub source_map: Vec<Span>,
}

#[derive(Default)]
pub struct Compiler {
    vars: ScopedMap<String, usize>,
    label_count: usize,
}

impl Compiler {
    pub fn compile(&mut self, expr: &Spanned<Expr>) -> Result<Program<Bytecode>, CompileError> {
        let program = self
            .compile_expr(expr)?
            .then_instruction(Stop, expr.1.end..expr.1.end);

        assert_eq!(program.instructions.len(), program.source_map.len());

        // TODO: Optimise the instuctions emitted by the above

        let bytecode_program = program.into_bytecode()?;

        Ok(bytecode_program)
    }

    fn compile_expr(&mut self, expr: &Spanned<Expr>) -> Result<Program<Instruction>, CompileError> {
        let instructions = match &expr.0 {
            Expr::Error => unreachable!(),

            Expr::Local(name) => self
                .compile_var_address(name, expr)?
                .then_instruction(Load, expr.1.clone()),

            Expr::Let(name, val) => self.compile_var_assign(expr, name, val)?,

            Expr::Value(AstValue::Func(func)) => {
                // TODO: Implement
                //   - [x] Static function calls (depends only on the function arguments)
                //   - [ ] Closures (depends on outer variables)
                // See https://craftinginterpreters.com/closures.html

                self.vars.start_scope();

                for (offset, arg) in func.args.iter().enumerate() {
                    self.vars.set_local(arg.clone(), offset);
                }

                let func_label = self.new_label();
                let post_func_label = self.new_label();

                let val = IrValue::Function(RuntimeFunction {
                    location: func_label,
                    arity: func.args.len(),
                });

                let program = Program::new()
                    .then_instructions(
                        vec![
                            Value(val),
                            Goto(post_func_label),
                            Instruction::Label(func_label),
                        ],
                        expr.1.clone(),
                    )
                    .then_program(self.compile_expr(&func.body)?)
                    .then_instructions(
                        vec![Return, Instruction::Label(post_func_label)],
                        expr.1.clone(),
                    );

                self.vars.pop_scope();

                program
            }

            Expr::Call(func, args) => {
                let func_program = self.compile_expr(func)?;

                args.iter()
                    .map(|arg| self.compile_expr(arg))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .fold(func_program, Program::then_program)
                    .then_instruction(Call(args.len()), expr.1.clone())
            }

            Expr::Value(val) => {
                let ir_val = IrValue::try_from(val).map_err(|msg| CompileError::Spanned {
                    span: expr.1.clone(),
                    msg,
                })?;

                Program::from_instruction(Value(ir_val), expr.1.clone())
            }

            Expr::Sequence(exprs) => {
                let mut program = exprs
                    .iter()
                    .map(|expr| self.compile_expr(expr))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .fold(Program::new(), |program, sub_program| {
                        let pop_span = sub_program.span().unwrap_or_else(|| expr.1.clone());

                        program
                            .then_program(sub_program)
                            // Everything is an expression, so values are left on the stack. For
                            // statement-style semi-colon-separated expressions, we pop the unused
                            // value left on the stack after each expression.
                            .then_instruction(Pop, pop_span)
                    });

                if !program.instructions.is_empty() {
                    // Only the last value in a sequence of expressions should be kept on the stack
                    program.pop_instruction();
                }

                program
            }

            Expr::Print(sub_expr) => self
                .compile_expr(sub_expr)?
                .then_instruction(PrintValue, expr.1.clone()),

            Expr::Block(sub_expr) => self.compile_expr(sub_expr)?,

            Expr::Unary(op, sub_expr) => {
                let program = self.compile_expr(sub_expr)?;

                let to_add = match op {
                    UnaryOp::Not => vec![Not],
                    UnaryOp::Neg => vec![Value(IrValue::Num(RuntimeNumber::Int(-1))), Mul],
                };

                program.then_instructions(to_add, expr.1.clone())
            }

            Expr::Binary(lhs, BinaryOp::And, rhs) => {
                let label_end = self.new_label();
                let label_false = self.new_label();

                Program::new()
                    .then_program(self.compile_expr(lhs)?)
                    .then_instruction(IfFalse(label_false), expr.1.clone())
                    .then_program(self.compile_expr(rhs)?)
                    .then_instructions(
                        vec![
                            Goto(label_end),
                            Instruction::Label(label_false),
                            Value(IrValue::Bool(false)),
                            Instruction::Label(label_end),
                        ],
                        expr.1.clone(),
                    )
            }

            Expr::Binary(lhs, BinaryOp::Or, rhs) => {
                let label_end = self.new_label();
                let label_true = self.new_label();

                Program::new()
                    .then_program(self.compile_expr(lhs)?)
                    .then_instruction(IfTrue(label_true), expr.1.clone())
                    .then_program(self.compile_expr(rhs)?)
                    .then_instructions(
                        vec![
                            Goto(label_end),
                            Instruction::Label(label_true),
                            Value(IrValue::Bool(true)),
                            Instruction::Label(label_end),
                        ],
                        expr.1.clone(),
                    )
            }

            Expr::Binary(lhs, op, rhs) => {
                let lhs_program = self.compile_expr(lhs)?;
                let rhs_program = self.compile_expr(rhs)?;

                let op_instr = match op {
                    BinaryOp::Add => Add,
                    BinaryOp::Sub => Sub,
                    BinaryOp::Mul => Mul,
                    BinaryOp::Div => Div,
                    BinaryOp::Eq => Eq,
                    BinaryOp::Less => Less,
                    BinaryOp::LessEq => LessEq,
                    BinaryOp::Greater => Greater,
                    BinaryOp::GreaterEq => GreaterEq,
                    _ => {
                        return Err(CompileError::Spanned {
                            span: expr.1.clone(),
                            msg: format!("Binary operator {:?} not implemented in compiler", op),
                        })
                    }
                };

                lhs_program
                    .then_program(rhs_program)
                    .then_instruction(op_instr, expr.1.clone())
            }

            Expr::List(items) => items
                .iter()
                .map(|item| self.compile_expr(item))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .fold(
                    Program::from_instruction(
                        Value(IrValue::List(IrList(Vec::new()))),
                        expr.1.clone(),
                    ),
                    |acc, p| {
                        acc.then_program(p)
                            .then_instruction(Method(Method::Append), expr.1.clone())
                    },
                ),

            Expr::If(cond, true_expr, false_expr) => {
                let cond_program = self.compile_expr(cond)?;
                let true_program = self.compile_expr(true_expr)?;
                let false_program = self.compile_expr(false_expr)?;

                let (false_label, end_label) = (self.new_label(), self.new_label());

                cond_program
                    .then_instruction(IfFalse(false_label), cond.1.clone())
                    .then_program(true_program)
                    .then_instruction(Goto(end_label), true_expr.1.clone())
                    .then_instruction(Instruction::Label(false_label), false_expr.1.clone())
                    .then_program(false_program)
                    .then_instruction(Instruction::Label(end_label), expr.1.clone())
            }

            _ => {
                return Err(CompileError::Spanned {
                    span: expr.1.clone(),
                    msg: format!("Compilation not implemented yet for {:?}", expr.0),
                })
            }
        };

        Ok(instructions)
    }

    // FIXME: The addresses here are completely nonsensical for outer scopes due to
    // base-pointer-relative addressing
    fn compile_var_address(
        &mut self,
        name: &String,
        expr: &Spanned<Expr>,
    ) -> Result<Program<Instruction>, CompileError> {
        // TODO: Upvalues / closures are not supported yet. Thus, only strictly local or global
        // variables are allowed.
        let var = self.vars.get(name).ok_or_else(|| CompileError::Spanned {
            span: expr.1.clone(),
            msg: format!("Variable {name} not found"),
        })?;

        let addr_instrs = match var {
            VarType::Local(offset) => {
                vec![GetBasePtr, ConstantInt(*offset as isize), Add]
            }
            VarType::Global(addr) => vec![ConstantInt(*addr as isize)],
        };

        Ok(Program::from_instructions(addr_instrs, expr.1.clone()))
    }

    // FIXME: Same as above
    fn compile_var_assign(
        &mut self,
        expr: &Spanned<Expr>,
        name: &String,
        val: &Spanned<Expr>,
    ) -> Result<Program<Instruction>, CompileError> {
        let mut program = Program::new();

        if self.vars.get(name).is_none() {
            // Allocate stack space for new local variable if it doesn't exist
            program.add_instruction(Value(IrValue::Null), expr.1.clone());

            let offset = self.vars.cur_scope_len();
            self.vars.set_local(name.clone(), offset);
        };

        Ok(program
            .then_program(self.compile_var_address(name, expr)?)
            .then_program(self.compile_expr(val)?)
            .then_instruction(Store, expr.1.clone()))
    }

    pub fn new_label(&mut self) -> Label {
        let label = Label(self.label_count);
        self.label_count += 1;
        label
    }
}

fn repeat_span(span: Span, count: usize) -> Vec<Span> {
    iter::repeat(span).take(count).collect()
}

impl<T> Program<T>
where
    T: std::fmt::Debug,
{
    pub fn new() -> Self {
        Program {
            instructions: Vec::new(),
            source_map: Vec::new(),
        }
    }

    pub fn disassemble(&self, src: &str) {
        for (pc, (instr, span)) in self.instructions.iter().zip(&self.source_map).enumerate() {
            let i = format!("{:?}", instr);
            let range = format!("{:?}", span);
            println!(
                "{:>3}: {:>20}  {:<8} {:?}",
                pc,
                i,
                range,
                &src[span.start..span.end]
            );
        }
    }

    pub fn from_instruction(instr: T, span: Span) -> Self {
        Program {
            source_map: vec![span],
            instructions: vec![instr],
        }
    }

    pub fn from_instructions(instrs: Vec<T>, span: Span) -> Self {
        Program {
            source_map: repeat_span(span, instrs.len()),
            instructions: instrs,
        }
    }

    pub fn add_instruction(&mut self, instr: T, span: Span) {
        self.source_map.push(span);
        self.instructions.push(instr);
    }

    pub fn add_instructions(&mut self, instrs: Vec<T>, span: Span) {
        self.source_map.extend(repeat_span(span, instrs.len()));
        self.instructions.extend(instrs);
    }

    pub fn then_instruction(mut self, instr: T, span: Span) -> Self {
        self.add_instruction(instr, span);
        self
    }

    pub fn then_instructions(mut self, instrs: Vec<T>, span: Span) -> Self {
        self.add_instructions(instrs, span);
        self
    }

    pub fn extend(&mut self, other: Self) {
        assert_eq!(self.instructions.len(), self.source_map.len());
        self.instructions.extend(other.instructions);
        self.source_map.extend(other.source_map);
    }

    pub fn then_program(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }

    pub fn pop_instruction(&mut self) -> Option<T> {
        self.source_map.pop();
        self.instructions.pop()
    }

    pub fn span(&self) -> Option<Span> {
        let start = self.source_map.iter().map(|s| s.start).min()?;
        let end = self.source_map.iter().map(|s| s.end).max()?;
        Some(Span { start, end })
    }
}

pub enum CompileError {
    Spanned { span: Span, msg: String },
    Plain(String),
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Label(pub usize);
