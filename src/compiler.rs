// TODO: Make all arguments generic/polymorphic, generate code for all possible types. Type inference.

use std::{collections::HashMap, iter};

use crate::{
    ast::{BinaryOp, Expr, Span, Spanned, UnaryOp, Value as AstValue},
    bytecode::Bytecode,
    ir_value::{IrList, IrValue},
    method::Method,
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
    Swap,
    GetStackPtr,
    SetStackPtr,

    // Binary operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
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
    Index,
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
    loop_count: usize,
    loop_labels: HashMap<usize, (Label, Label)>,
}

impl Compiler {
    pub fn compile(&mut self, expr: &Spanned<Expr>) -> Result<Program<Bytecode>, CompileError> {
        let program = self
            .compile_expr(expr)?
            .then_instruction(Stop, expr.1.end..expr.1.end);

        assert_eq!(program.instructions.len(), program.source_map.len());

        // TODO: Optimise the instuctions emitted by the above
        //  - [ ] Remove unnecessary additions
        //  - [ ] Don't do lookups on constants, just insert them

        let bytecode_program = program.into_bytecode()?;

        Ok(bytecode_program)
    }

    fn compile_expr(&mut self, expr: &Spanned<Expr>) -> Result<Program<Instruction>, CompileError> {
        let instructions = match &expr.0 {
            Expr::Local(name) => self.compile_var_load(expr, name)?,

            Expr::Let(name, val) => {
                let val_program = self.compile_expr(val)?;
                self.compile_var_assign(expr, name, val_program)?
            }

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

            Expr::Return(val) => {
                if self.vars.is_currently_top_scope() {
                    return Err(CompileError::Spanned {
                        span: expr.1.clone(),
                        msg: "Illegal return outside of function".to_string(),
                    });
                }

                self.compile_expr(val)?
                    .then_instruction(Return, expr.1.clone())
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
                    BinaryOp::Mod => Mod,
                    BinaryOp::Eq => Eq,
                    BinaryOp::NotEq => NotEq,
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

            Expr::Index(value, index) => {
                let value_program = self.compile_expr(value)?;
                let index_program = self.compile_expr(index)?;

                value_program
                    .then_program(index_program)
                    .then_instruction(Index, index.1.clone())
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

            Expr::While(cond, body) => {
                let (cond_label, end_label) = (self.new_label(), self.new_label());

                let loop_id = self.new_loop_id();
                let loop_name = self.loop_name(loop_id);
                self.loop_labels.insert(loop_id, (cond_label, end_label));
                let register_loop = self
                    .compile_var_assign(
                        expr,
                        &loop_name,
                        Program::from_instructions(vec![GetStackPtr], expr.1.clone()),
                    )?
                    .then_instruction(Pop, expr.1.clone());

                let program = register_loop
                    // result of last iteration: null if no iterations or popped and replaced by
                    // upcoming iterations
                    .then_instruction(Value(IrValue::Null), expr.1.clone())
                    .then_instruction(Instruction::Label(cond_label), expr.1.clone())
                    .then_program(self.compile_expr(cond)?)
                    .then_instruction(IfFalse(end_label), cond.1.clone())
                    .then_program(self.compile_expr(body)?)
                    // last expression in the block will leave a new value on the stack, so pop
                    // the current "last value" off
                    .then_instructions(vec![Swap, Pop, Goto(cond_label)], expr.1.clone())
                    .then_instructions(
                        vec![Instruction::Label(end_label), Swap, Pop],
                        expr.1.clone(),
                    );

                self.vars.remove_local(&loop_name);

                program
            }

            Expr::Break => self.compile_loop_jump("break", expr, |(_, end_label)| end_label)?,
            Expr::Continue => {
                self.compile_loop_jump("continue", expr, |(cond_label, _)| cond_label)?
            }

            Expr::MethodCall(target, method_name, args) => {
                let target_program = self.compile_expr(target)?;

                let method_instr =
                    Method::from_name(method_name).ok_or_else(|| CompileError::Spanned {
                        span: expr.1.clone(),
                        msg: format!("Method {method_name:?} is unknown"),
                    })?;

                let program = args
                    .iter()
                    .map(|arg| self.compile_expr(arg))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .fold(target_program, Program::then_program);

                // TODO: pass along how many args were given
                program.then_instruction(Method(method_instr), expr.1.clone())
            }

            Expr::ParseError => {
                return Err(CompileError::Spanned {
                    msg: "Parse error".to_string(),
                    span: expr.1.clone(),
                })
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
            msg: format!("No such variable '{name}' in scope"),
        })?;

        let addr_instrs = match var {
            VarType::Local(offset) => {
                vec![GetBasePtr, ConstantInt(*offset as isize), Add]
            }
            VarType::Global(addr) => vec![ConstantInt(*addr as isize)],
        };

        Ok(Program::from_instructions(addr_instrs, expr.1.clone()))
    }

    fn compile_var_load(
        &mut self,
        expr: &Spanned<Expr>,
        name: &String,
    ) -> Result<Program<Instruction>, CompileError> {
        Ok(self
            .compile_var_address(name, expr)?
            .then_instruction(Load, expr.1.clone()))
    }

    // FIXME: Same as above
    fn compile_var_assign(
        &mut self,
        expr: &Spanned<Expr>,
        name: &String,
        value_program: Program<Instruction>,
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
            .then_program(value_program)
            .then_instruction(Store, expr.1.clone()))
    }

    pub fn new_label(&mut self) -> Label {
        let label = Label(self.label_count);
        self.label_count += 1;
        label
    }

    pub fn new_loop_id(&mut self) -> usize {
        let loop_id = self.loop_count;
        self.loop_count += 1;
        loop_id
    }

    pub fn loop_name(&self, id: usize) -> String {
        format!("!loop_{id}")
    }

    pub fn local_loop_vars(&self) -> impl Iterator<Item = (&String, &usize)> {
        self.vars
            .iter_local()
            .filter(|(name, _)| name.starts_with("!loop_"))
    }

    pub fn is_in_loop(&mut self) -> bool {
        self.local_loop_vars().next().is_some()
    }

    pub fn cur_loop_id(&self) -> usize {
        self.local_loop_vars()
            .max_by_key(|(_, offset)| **offset)
            .map(|(name, _)| {
                name.strip_prefix("!loop_")
                    .unwrap()
                    .parse()
                    .expect("loop name is not a number")
            })
            .expect("not in a loop")
    }

    // 1. Get the current loop name
    // 2. Set stack pointer to that number
    // 3. Swap top of stack [sp, last_val] -> [last_val, sp]
    // 4. Pop the top of the stack
    pub fn compile_loop_jump(
        &mut self,
        action: &str,
        expr: &Spanned<Expr>,
        get_jump_to: impl FnOnce((Label, Label)) -> Label,
    ) -> Result<Program<Instruction>, CompileError> {
        if !self.is_in_loop() {
            return Err(CompileError::Spanned {
                span: expr.1.clone(),
                msg: format!("Cannot {action} outside of loop"),
            });
        }

        let loop_id = self.cur_loop_id();
        let loop_name = self.loop_name(loop_id);
        let (cond_label, end_label) = *self
            .loop_labels
            .get(&loop_id)
            .expect("labels for loop id not found");
        let jump_to = get_jump_to((cond_label, end_label));

        Ok(self
            .compile_var_load(expr, &loop_name)?
            .then_instructions(vec![SetStackPtr, Goto(jump_to)], expr.1.clone()))
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

            let source = if span.end <= span.start + 30 {
                src[span.start..span.end].to_string()
            } else {
                format!("{}...", &src[span.start..(span.start + 30)])
            };

            println!("{pc:>3}: {i:>20}  {range:<8} {source:?}");
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
