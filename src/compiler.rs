// TODO: Make all arguments generic/polymorphic, generate code for all possible types. Type inference.

use std::iter;

use crate::{
    ast::{BinaryOp, Expr, Span, Spanned, UnaryOp, Value as AstValue},
    bytecode::Bytecode,
    ir_value::{IrList, IrValue},
    runtime_value::function::RuntimeFunction,
    scoped_map::ScopedMap,
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

            Expr::Value(AstValue::Func(func)) => {
                // TODO: Implement
                //   - [x] Static function calls (depends only on the function arguments)
                //   - [ ] Closures (depends on outer variables)
                // See https://craftinginterpreters.com/closures.html

                self.vars.start_scope();

                for (offset, arg) in func.args.iter().enumerate() {
                    self.vars.set(arg.clone(), offset);
                }

                let func_label = self.new_label();
                let post_func_label = self.new_label();

                let val = IrValue::Function(RuntimeFunction {
                    location: func_label,
                    arity: func.args.len(),
                });

                let program = Program::new()
                    .then_instruction(Value(val), expr.1.clone())
                    .then_instruction(Goto(post_func_label), expr.1.clone())
                    .then_instruction(Instruction::Label(func_label), expr.1.clone())
                    .then_program(self.compile_expr(&func.body)?)
                    .then_instruction(Return, expr.1.clone())
                    .then_instruction(Instruction::Label(post_func_label), expr.1.clone());

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

                Program::new().then_instruction(Value(ir_val), expr.1.clone())
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
                            // statement-style semi-colon-separated expressions, we pop the value
                            // left on the stack after each expression.
                            // FIXME: This is buggy. A series of variable assignments results in
                            // the variables being popped off the stack. But how the hell do we
                            // handle popping in the correct instances and not popping in the
                            // incorrect instances? Idea: make a "statement" - anything with a
                            // semicolon after is wrapped in a statement. Would this work? VM would
                            // still push stuff to stack, though. Need to make compiler emit a
                            // push-stack instruction?
                            .then_instruction(Pop, pop_span)
                    });

                if !program.instructions.is_empty() {
                    // Only the last value in a sequence of expressions should be kept on the stack
                    program.pop();
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
                    UnaryOp::Neg => vec![ConstantInt(-1), Mul],
                };

                program.then_instructions(to_add, expr.1.clone())
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
                    _ => {
                        return Err(CompileError::Spanned {
                            span: expr.1.clone(),
                            msg: format!("Binary operator {:?} not implemented", op),
                        })
                    }
                };

                lhs_program
                    .then_program(rhs_program)
                    .then_instruction(op_instr, expr.1.clone())
            }

            Expr::List(items) => {
                let initial_val = Program::new()
                    .then_instruction(Value(IrValue::List(IrList(Vec::new()))), expr.1.clone());

                items
                    .iter()
                    .map(|item| self.compile_expr(item))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .fold(initial_val, |acc, p| {
                        acc.then_program(p)
                            .then_instruction(Method(Method::Append), expr.1.clone())
                    })
            }

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
    fn compile_var_access(&mut self, name: &String) -> Result<Vec<Instruction>, String> {
        let addr = self
            .vars
            // TODO: Upvalues / closures are not supported yet
            .get_local(name)
            .ok_or_else(|| format!("Variable {name} not found"))?;

        Ok(vec![GetBasePtr, ConstantInt(*addr as isize), Add, Load])
    }

    // FIXME: Same as above
    fn compile_var_assign(
        &mut self,
        expr: &Spanned<Expr>,
        name: &String,
        val: &Spanned<Expr>,
    ) -> Result<Program<Instruction>, CompileError> {
        let mut program = Program::new();

        let addr = self.vars.get_local(name).cloned().unwrap_or_else(|| {
            // Allocate space for new local variable if it doesn't exist
            program.add_instruction(Value(IrValue::Null), expr.1.clone());

            let local_addr = self.vars.cur_scope_len();
            self.vars.set(name.clone(), local_addr);
            local_addr
        });

        Ok(program
            .then_instructions(
                vec![GetBasePtr, ConstantInt(addr as isize), Add],
                expr.1.clone(),
            )
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

    pub fn pop(&mut self) -> Option<T> {
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
