// TODO: Make all arguments generic/polymorphic, generate code for all possible types. Type inference.

use std::{collections::HashMap, ops::RangeInclusive};

use crate::{
    compiler::{
        ir_value::IrValue,
        method::Method,
        scoped_map::{ScopedMap, VarType},
        stdlib_fn::StdlibFn,
    },
    grammar::ast::{AstValue, BinaryOp, Expr, Pattern, Span, Spanned, UnaryOp},
    vm::{
        bytecode::Bytecode,
        runtime_value::{function::RuntimeFunction, number::RuntimeNumber},
    },
};

pub mod analysis;
pub mod ir_value;
pub mod method;
pub mod register_manager;
pub mod scoped_map;
pub mod stdlib_fn;

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
    RemoveIndex,
    Swap,
    Dup,
    GetStackPtr,
    SetStackPtr,

    // Register manipulation
    SetRegister(usize),
    GetRegister(usize),

    // Binary operations
    Add,
    Sub,
    Mul,
    Div,
    DivFloor,
    Mod,
    Pow,
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Range,
    Xor,
    BitwiseAnd,

    // Unary operations
    Not,

    // Control flow
    Stop,
    Goto(Label),
    Label(Label),
    IfTrue(Label),
    IfFalse(Label),
    RuntimeError(String),

    // Functions
    GetBasePtr,
    Call(usize),
    Return,

    // Standard library functions and built-ins
    StdlibCall(StdlibFn, usize),
    MethodCall(Method, usize),
    IsIn,
    Index,
    SetIndex,
    NextIter,
    ToIter,
    CreateTuple(usize),
}

use chumsky::span::Span as _;
use Instruction::*;

type LoopId = Span;

#[derive(Debug, Default)]
pub struct Program<T> {
    pub instructions: Vec<T>,
    pub source_map: Vec<Span>,
}

#[derive(Default)]
pub struct Compiler {
    vars: ScopedMap<String, usize>,
    registers: register_manager::RegisterManager,
    label_count: usize,
    loop_labels: HashMap<LoopId, (Label, Label)>,
    loop_stack: Vec<LoopId>,
}

impl Compiler {
    pub fn compile(&mut self, expr: &Spanned<Expr>) -> Result<Program<Bytecode>, CompileError> {
        let program = self
            .compile_allocation_for_all_vars_in_scope(expr)
            .then_program(self.compile_expr(expr)?)
            .then_instruction(Stop, expr.span().to_end());

        assert_eq!(program.instructions.len(), program.source_map.len());

        // TODO: Optimise the instuctions emitted by the above
        //  - [ ] Remove unnecessary additions
        //  - [ ] Don't do lookups on constants, just insert them

        let bytecode_program = program.into_bytecode()?;

        Ok(bytecode_program)
    }

    fn compile_expr(&mut self, expr: &Spanned<Expr>) -> Result<Program<Instruction>, CompileError> {
        if let Some(constant) =
            analysis::eval_simple_constant(expr).map_err(|msg| CompileError::Spanned {
                span: expr.span(),
                msg,
            })?
        {
            return Ok(Program::from_instruction(Value(constant), expr.span()));
        }

        let instructions = match &expr.0 {
            Expr::Local(name) => self.compile_var_load(expr, name)?,

            Expr::Assign(pattern, val) => self
                .compile_expr(val)?
                .then_program(self.compile_pattern_assignment(expr, pattern)?),

            Expr::Value(AstValue::Func(func)) => {
                // TODO: Implement
                //   - [x] Static function calls (depends only on the function arguments)
                //   - [ ] Closures (depends on outer variables)
                // See https://craftinginterpreters.com/closures.html

                self.vars.start_scope();

                for (offset, arg) in func.args.iter().enumerate() {
                    self.vars.set_local(arg.to_string(), offset);
                }

                let func_label = self.new_label();
                let post_func_label = self.new_label();

                let val = IrValue::Function(RuntimeFunction {
                    location: func_label,
                    arity: func.args.len(),
                    is_memoized: func.is_memoized,
                });

                let program = Program::new()
                    .then_instructions(
                        vec![
                            Value(val),
                            Goto(post_func_label),
                            Instruction::Label(func_label),
                        ],
                        expr.span(),
                    )
                    .then_program(self.compile_allocation_for_all_vars_in_scope(&func.body))
                    .then_program(self.compile_expr(&func.body)?)
                    .then_instructions(
                        vec![Return, Instruction::Label(post_func_label)],
                        expr.span(),
                    );

                self.vars.pop_scope();

                program
            }

            Expr::Call(func, args) => {
                if let Expr::Local(name) = &func.0 {
                    if let Some(stdlib_fn) = StdlibFn::from_name(name) {
                        return self.compile_stdlib_call(stdlib_fn, args, expr);
                    }
                }

                let func_program = self.compile_expr(func)?;

                args.iter()
                    .map(|arg| self.compile_expr(arg))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .fold(func_program, Program::then_program)
                    .then_instruction(Call(args.len()), expr.span())
            }

            Expr::Return(val) => {
                if self.vars.is_currently_top_scope() {
                    return Err(CompileError::Spanned {
                        span: expr.span(),
                        msg: "Illegal return outside of function".to_string(),
                    });
                }

                self.compile_expr(val)?
                    .then_instruction(Return, expr.span())
            }

            Expr::Value(val) => {
                let ir_val = IrValue::try_from(val).map_err(|msg| CompileError::Spanned {
                    span: expr.span(),
                    msg,
                })?;

                Program::from_instruction(Value(ir_val), expr.span())
            }

            Expr::Sequence(exprs) => {
                let mut program = exprs
                    .iter()
                    .map(|expr| self.compile_expr(expr))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .fold(Program::new(), |program, sub_program| {
                        let pop_span = sub_program.span().unwrap_or_else(|| expr.span());

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

            Expr::Block(sub_expr) => self.compile_expr(sub_expr)?,

            Expr::Unary(op, sub_expr) => {
                let program = self.compile_expr(sub_expr)?;

                let to_add = match op {
                    UnaryOp::Not => vec![Not],
                    UnaryOp::Neg => vec![Value(IrValue::Num(RuntimeNumber::from(-1))), Mul],
                };

                program.then_instructions(to_add, expr.span())
            }

            Expr::Binary(lhs, BinaryOp::And, rhs) => {
                let label_end = self.new_label();
                let label_false = self.new_label();

                Program::new()
                    .then_program(self.compile_expr(lhs)?)
                    .then_instruction(IfFalse(label_false), expr.span())
                    .then_program(self.compile_expr(rhs)?)
                    .then_instructions(
                        vec![
                            Goto(label_end),
                            Instruction::Label(label_false),
                            Value(IrValue::Bool(false)),
                            Instruction::Label(label_end),
                        ],
                        expr.span(),
                    )
            }

            Expr::Binary(lhs, BinaryOp::Or, rhs) => {
                let label_end = self.new_label();

                Program::new()
                    .then_program(self.compile_expr(lhs)?)
                    .then_instruction(Dup, expr.span())
                    .then_instruction(IfTrue(label_end), expr.span())
                    .then_instruction(Pop, expr.span())
                    .then_program(self.compile_expr(rhs)?)
                    .then_instruction(Instruction::Label(label_end), expr.span())
            }

            Expr::Binary(lhs, op, rhs) => {
                let lhs_program = self.compile_expr(lhs)?;
                let rhs_program = self.compile_expr(rhs)?;

                let op_instr = match op {
                    BinaryOp::Add => Add,
                    BinaryOp::Sub => Sub,
                    BinaryOp::Mul => Mul,
                    BinaryOp::Div => Div,
                    BinaryOp::DivFloor => DivFloor,
                    BinaryOp::Mod => Mod,
                    BinaryOp::Pow => Pow,
                    BinaryOp::Eq => Eq,
                    BinaryOp::NotEq => NotEq,
                    BinaryOp::Less => Less,
                    BinaryOp::LessEq => LessEq,
                    BinaryOp::Greater => Greater,
                    BinaryOp::GreaterEq => GreaterEq,
                    BinaryOp::Range => Range,
                    BinaryOp::Xor => Xor,
                    BinaryOp::In => IsIn,
                    BinaryOp::BitwiseAnd => BitwiseAnd,
                    _ => {
                        return Err(CompileError::Spanned {
                            span: expr.span(),
                            msg: format!("Binary operator {:?} not implemented in compiler", op),
                        })
                    }
                };

                lhs_program
                    .then_program(rhs_program)
                    .then_instruction(op_instr, expr.span())
            }

            Expr::List(items) => items.iter().try_fold(
                Program::from_instruction(Value(IrValue::new_list()), expr.span()),
                |acc, item| {
                    Ok(acc
                        .then_program(self.compile_expr(item)?)
                        .then_instruction(MethodCall(Method::Append, 1), expr.span()))
                },
            )?,

            Expr::Tuple(items) => {
                // Compile each tuple element onto the stack
                let program = items.iter().try_fold(Program::new(), |acc, item| {
                    Ok(acc.then_program(self.compile_expr(item)?))
                })?;

                // Emit CreateTuple instruction to consume N values from stack
                program.then_instruction(CreateTuple(items.len()), expr.span())
            }

            Expr::Map(items) => items.iter().try_fold(
                Program::from_instruction(Value(IrValue::new_map()), expr.span()),
                |acc, (key, value)| {
                    Ok(acc
                        .then_program(self.compile_expr(key)?)
                        .then_program(self.compile_expr(value)?)
                        .then_instruction(SetIndex, expr.span()))
                },
            )?,

            Expr::Index(value, index) => {
                let value_program = self.compile_expr(value)?;
                let index_program = self.compile_expr(index)?;

                value_program
                    .then_program(index_program)
                    .then_instruction(Index, index.span())
            }

            Expr::If(cond, true_expr, false_expr) => {
                let cond_program = self.compile_expr(cond)?;
                let true_program = self.compile_expr(true_expr)?;
                let false_program = self.compile_expr(false_expr)?;

                let (false_label, end_label) = (self.new_label(), self.new_label());

                cond_program
                    .then_instruction(IfFalse(false_label), cond.span())
                    .then_program(true_program)
                    .then_instruction(Goto(end_label), true_expr.span())
                    .then_instruction(Instruction::Label(false_label), false_expr.span())
                    .then_program(false_program)
                    .then_instruction(Instruction::Label(end_label), expr.span())
            }

            // For an explanation of the stack layout for while loops, see the comment below for
            // for loops. The only difference is that no iterator is needed, so the stack pointer
            // is only added with 1 (only 1 tmp variable).
            Expr::While(cond, body) => {
                let (cond_label, end_label) = (self.new_label(), self.new_label());

                let loop_vars = make_loop_vars(expr.span());

                self.loop_labels
                    .insert(loop_vars.id, (cond_label, end_label));

                self.loop_stack.push(loop_vars.id);

                let register_loop = self
                    .compile_var_assign(
                        expr,
                        &loop_vars.stack_ptr_var,
                        Program::from_instructions(
                            vec![GetStackPtr, ConstantInt(1), Add],
                            expr.span(),
                        ),
                    )?
                    .then_instruction(Pop, expr.span());

                let program = register_loop
                    .then_instruction(Value(IrValue::Null), expr.span())
                    .then_instruction(Instruction::Label(cond_label), expr.span())
                    .then_program(self.compile_expr(cond)?)
                    .then_instruction(IfFalse(end_label), cond.span())
                    .then_program(self.compile_expr(body)?)
                    .then_instructions(vec![Swap, Pop, Goto(cond_label)], expr.span())
                    .then_instructions(vec![Instruction::Label(end_label)], expr.span());

                self.loop_stack.pop();

                program
            }

            // The stack layout for a for loop is as follows:
            //    Initialisation:     OLD_SP  ITERATOR  null
            //    First iteration:    OLD_SP  ITERATOR  null  LAST_EXPR
            //    Cleanup first:      OLD_SP  ITERATOR  LAST_EXPR
            //    Cleanup loop:       LAST_EXPR
            //
            // So, to initialise:
            //    1. (the loop variable is already allocated at the start of the function)
            //    2. Allocate tmp OLD_SP, the stack pointer at the start of the loop (for continue/break)
            //       - The stack pointer should point to the "null" position above.
            //    3. Allocate tmp ITERATOR, the iterator for the loop
            //    4. Place an output value on the stack, initially null in case of no iterations
            //
            // At the end of each iteration, replace the "last value" with the new value:
            //    1. Just swap, pop
            //
            // To finalise loop and clean up temporary variables:
            //    1. Fix the stack, discarding OLD_SP and ITERATOR:
            //      - Swap, pop, swap, pop
            //    2. Fix compiler variable state un-register variables for OLD_SP, ITERATOR, and OUTPUT in scope
            //
            // For all this, it is crucial that all variables in scope are pre-allocated!
            // Otherwise, the top of the stack is messed up by variables allocated inside the loop.
            //
            // To perform break/continue, simply truncate the stack to after ITERATOR (thus
            // discarding all local state after the iteration was started), then jumping to either
            // the next iteration or the end of the loop.
            Expr::For(loop_var, iterable, body) => {
                let (iter_label, end_label) = (self.new_label(), self.new_label());

                let scope_size_before = self.vars.cur_scope_len();

                let loop_vars = make_loop_vars(expr.span());

                self.loop_labels
                    .insert(loop_vars.id, (iter_label, end_label));

                self.loop_stack.push(loop_vars.id);

                let register_loop = self
                    .compile_var_assign(
                        expr,
                        &loop_vars.stack_ptr_var,
                        Program::from_instructions(
                            vec![GetStackPtr, Value(IrValue::Int(1)), Add],
                            expr.span(),
                        ),
                    )?
                    .then_instruction(Pop, expr.span());

                let iterator = self
                    .compile_expr(iterable)?
                    .then_instruction(ToIter, iterable.span());
                let register_iterable = self
                    .compile_var_assign(expr, &loop_vars.iterator_var, iterator)?
                    .then_instruction(Pop, iterable.span());

                let program = register_loop
                    .then_program(register_iterable)
                    .then_instruction(Value(IrValue::Null), expr.span())
                    .then_instruction(Instruction::Label(iter_label), expr.span())
                    .then_program(self.compile_var_load(expr, &loop_vars.iterator_var)?)
                    .then_instructions(vec![NextIter, IfFalse(end_label)], expr.span())
                    .then_program(self.compile_loop_var_assign(loop_var, expr)?)
                    .then_program(self.compile_expr(body)?)
                    .then_instructions(vec![Swap, Pop, Goto(iter_label)], expr.span())
                    .then_instruction(Instruction::Label(end_label), expr.span());

                self.loop_stack.pop();

                debug_assert!(
                    self.vars.cur_scope_len() == scope_size_before,
                    "Variables were left on the stack within loop"
                );

                program
            }

            Expr::Break => self.compile_loop_jump("break", expr, |(_, end_label)| end_label)?,
            Expr::Continue => {
                self.compile_loop_jump("continue", expr, |(cond_label, _)| cond_label)?
            }

            Expr::ListComprehension(body, loop_var, iterable) => {
                let (iter_label, end_label) = (self.new_label(), self.new_label());

                let scope_size_before = self.vars.cur_scope_len();

                let loop_vars = make_loop_vars(expr.span());

                self.loop_labels
                    .insert(loop_vars.id, (iter_label, end_label));

                self.loop_stack.push(loop_vars.id);

                let register_loop = self
                    .compile_var_assign(
                        expr,
                        &loop_vars.stack_ptr_var,
                        Program::from_instructions(
                            vec![GetStackPtr, ConstantInt(1), Add],
                            expr.span(),
                        ),
                    )?
                    .then_instruction(Pop, expr.span());

                let iterator = self
                    .compile_expr(iterable)?
                    .then_instruction(ToIter, iterable.span());
                let register_iterable = self
                    .compile_var_assign(expr, &loop_vars.iterator_var, iterator)?
                    .then_instruction(Pop, iterable.span());

                let program = register_loop
                    .then_program(register_iterable)
                    .then_instruction(Value(IrValue::List(Vec::new())), expr.span())
                    .then_instruction(Instruction::Label(iter_label), expr.span())
                    .then_program(self.compile_var_load(expr, &loop_vars.iterator_var)?)
                    .then_instructions(vec![NextIter, IfFalse(end_label)], expr.span())
                    .then_program(self.compile_loop_var_assign(loop_var, expr)?)
                    .then_program(self.compile_expr(body)?)
                    .then_instructions(
                        vec![MethodCall(Method::Append, 1), Goto(iter_label)],
                        expr.span(),
                    )
                    .then_instruction(Instruction::Label(end_label), expr.span());

                self.loop_stack.pop();

                debug_assert!(
                    self.vars.cur_scope_len() == scope_size_before,
                    "Variables were left on the stack within loop"
                );

                program
            }

            Expr::MethodCall(target, method_name, args) => {
                let target_program = self.compile_expr(target)?;

                let method =
                    Method::from_name(method_name).ok_or_else(|| CompileError::Spanned {
                        span: expr.span(),
                        msg: format!("Method {method_name:?} is unknown"),
                    })?;

                if let Err(msg) = validate_num_args(method.num_args(), args.len()) {
                    return Err(CompileError::Spanned {
                        span: expr.span(),
                        msg: format!("Method {} {msg}", method.name()),
                    });
                }

                let program = args
                    .iter()
                    .map(|arg| self.compile_expr(arg))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .fold(target_program, Program::then_program);

                program.then_instruction(MethodCall(method, args.len()), expr.span())
            }

            Expr::Match(val, arms) => {
                let mut program = self.compile_expr(val)?;

                let labels = arms.iter().map(|_| self.new_label()).collect::<Vec<_>>();
                let (label_last, label_end) = (self.new_label(), self.new_label());

                for (i, (pattern, body)) in arms.iter().enumerate() {
                    let constant_opt = analysis::eval_simple_constant(pattern).map_err(|msg| {
                        CompileError::Spanned {
                            span: expr.span(),
                            msg,
                        }
                    })?;

                    let cur_label = labels[i];
                    let next_label = labels.get(i + 1).copied().unwrap_or(label_last);

                    // TODO: Implement more advanced constant types (e.g. lists, tuples, sets)
                    // TODO: Implement catch-all (ident => body)
                    // TODO: Implement pattern matching (incl. binding values like (x, y) => x + y)
                    if let Some(constant) = constant_opt {
                        let arm_program = Program::from_instructions(
                            vec![
                                Instruction::Label(cur_label),
                                Dup,
                                Value(constant),
                                Eq,
                                IfFalse(next_label),
                            ],
                            pattern.span(),
                        )
                        .then_program(self.compile_expr(body)?)
                        .then_instruction(Goto(label_end), expr.span());

                        program.extend(arm_program);
                    } else {
                        return Err(CompileError::Spanned {
                            span: expr.span(),
                            msg: "Pattern matching not implemented yet".to_string(),
                        });
                    }
                }

                program.add_instructions(
                    vec![
                        Instruction::Label(label_last),
                        RuntimeError("No arm matched the value".to_string()),
                        Instruction::Label(label_end),
                    ],
                    expr.span(),
                );

                program
            }

            Expr::ParseError => {
                return Err(CompileError::Spanned {
                    msg: "Parse error".to_string(),
                    span: expr.span(),
                })
            }

            #[allow(unreachable_patterns)]
            _ => {
                return Err(CompileError::Spanned {
                    span: expr.span(),
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
        name: &str,
        expr: &Spanned<Expr>,
    ) -> Result<Program<Instruction>, CompileError> {
        // TODO: Upvalues / closures are not supported yet. Thus, only strictly local or global
        // variables are allowed.
        let key = name.to_string();
        let var = self.vars.get(&key).ok_or_else(|| CompileError::Spanned {
            span: expr.span(),
            msg: format!("No such variable '{name}' in scope"),
        })?;

        let addr_instrs = match var {
            VarType::Local(offset) => {
                vec![GetBasePtr, ConstantInt(*offset as isize), Add]
            }
            VarType::Global(addr) => vec![ConstantInt(*addr as isize)],
        };

        Ok(Program::from_instructions(addr_instrs, expr.span()))
    }

    fn compile_var_load(
        &mut self,
        expr: &Spanned<Expr>,
        name: &str,
    ) -> Result<Program<Instruction>, CompileError> {
        Ok(self
            .compile_var_address(name, expr)?
            .then_instruction(Load, expr.span()))
    }

    fn compile_var_assign(
        &mut self,
        expr: &Spanned<Expr>,
        name: &str,
        value_program: Program<Instruction>,
    ) -> Result<Program<Instruction>, CompileError> {
        let key = name.to_string();
        if self.vars.get(&key).is_none() {
            return Err(CompileError::Spanned {
                msg: format!(
                    "Internal compiler bug: allocation for variable {name:?} should have been done before assignment"
                ),
                span: expr.span(),
            });
        };

        Ok(value_program
            .then_program(self.compile_var_address(name, expr)?)
            .then_instruction(Store, expr.span()))
    }

    fn compile_allocation_for_all_vars_in_scope(
        &mut self,
        expr: &Spanned<Expr>,
    ) -> Program<Instruction> {
        analysis::find_all_assignments(expr).into_iter().fold(
            Program::new(),
            |program, assignment| {
                if self.vars.get(&assignment).is_some() {
                    return program;
                }

                self.vars
                    .set_local(assignment.to_string(), self.vars.cur_scope_len());
                program.then_instruction(Value(IrValue::Uninit), assignment.span())
            },
        )
    }

    // Assumes that the value is currently on top of the stack.
    pub fn compile_pattern_assignment(
        &mut self,
        expr: &Spanned<Expr>,
        pattern: &Spanned<Pattern>,
    ) -> Result<Program<Instruction>, CompileError> {
        let prog = match &pattern.0 {
            Pattern::Ident(name) => self
                .compile_var_address(name, expr)?
                .then_instruction(Store, pattern.span()),

            Pattern::Sequence(patterns) => patterns
                .iter()
                .map(|p| self.compile_pattern_assignment(expr, p))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .enumerate()
                .fold(Program::new(), |program, (i, p)| {
                    let index = Value(IrValue::Num(RuntimeNumber::from(i as isize)));
                    program
                        .then_instructions(vec![Dup, index, Index], expr.span())
                        .then_program(p)
                        .then_instruction(Pop, expr.span())
                }),

            Pattern::Index(target, index) => {
                let val_addr_register = self.get_available_register(expr.span())?;

                let target_program = self.compile_expr(target)?;
                let index_program = self.compile_expr(index)?;

                let program = Program::from_instructions(
                    vec![GetStackPtr, SetRegister(val_addr_register)],
                    pattern.span(),
                )
                .then_program(target_program)
                .then_program(index_program)
                .then_instructions(vec![GetRegister(val_addr_register), Load], pattern.span())
                .then_instruction(SetIndex, pattern.span())
                .then_instruction(Pop, pattern.span());

                self.registers.free_register(val_addr_register);

                program
            }

            _ => {
                return Err(CompileError::Spanned {
                    span: pattern.span(),
                    msg: "Pattern matching not implemented yet".to_string(),
                })
            }
        };

        Ok(prog)
    }

    fn compile_loop_var_assign(
        &mut self,
        pattern: &Spanned<Pattern>,
        expr: &Spanned<Expr>,
    ) -> Result<Program<Instruction>, CompileError> {
        let prog = self
            .compile_pattern_assignment(expr, pattern)?
            .then_instruction(Pop, pattern.span());

        Ok(prog)
    }

    pub fn get_available_register(&mut self, span: Span) -> Result<usize, CompileError> {
        self.registers
            .get_available_register()
            .ok_or_else(|| CompileError::Spanned {
                span,
                msg: "The program requires too many registers".to_string(),
            })
    }

    pub fn new_label(&mut self) -> Label {
        let label = Label(self.label_count);
        self.label_count += 1;
        label
    }

    pub fn is_in_loop(&mut self) -> bool {
        self.loop_stack
            .last()
            .filter(|loop_id| {
                let loop_vars = make_loop_vars(**loop_id);
                self.vars.get_local(&loop_vars.stack_ptr_var).is_some()
            })
            .is_some()
    }

    pub fn cur_loop_id(&self) -> LoopId {
        *self.loop_stack.last().expect("not in a loop")
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
                span: expr.span(),
                msg: format!("Cannot {action} outside of loop"),
            });
        }

        let loop_id = self.cur_loop_id();
        let loop_vars = make_loop_vars(loop_id);
        let (cond_label, end_label) = *self
            .loop_labels
            .get(&loop_id)
            .expect("labels for loop id not found");
        let jump_to = get_jump_to((cond_label, end_label));

        Ok(self
            .compile_var_load(expr, &loop_vars.stack_ptr_var)?
            .then_instructions(vec![SetStackPtr, Goto(jump_to)], expr.span()))
    }

    fn compile_stdlib_call(
        &mut self,
        stdlib_fn: StdlibFn,
        args: &[Spanned<Expr>],
        expr: &Spanned<Expr>,
    ) -> Result<Program<Instruction>, CompileError> {
        if let Err(msg) = validate_num_args(stdlib_fn.num_args(), args.len()) {
            return Err(CompileError::Spanned {
                span: expr.span(),
                msg: format!("Function {} {msg}", stdlib_fn.name()),
            });
        }

        let program = args
            .iter()
            .map(|arg| self.compile_expr(arg))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .fold(Program::new(), Program::then_program);

        Ok(program.then_instruction(StdlibCall(stdlib_fn, args.len()), expr.span()))
    }
}

pub struct LoopVars {
    pub stack_ptr_var: String,
    pub iterator_var: String,
    pub name: String,
    pub id: LoopId,
}

// This is an easy way to standardise loop variable names without collisions (since loops
// should be unique in the source code) and without adding synchronized logic between analysis
// and the compiler.
fn make_loop_vars(span: LoopId) -> LoopVars {
    let loop_id = format!("{}..{}", span.start, span.end);
    let loop_name = format!("!loop_{}", loop_id);
    LoopVars {
        stack_ptr_var: format!("{loop_name}_sp"),
        iterator_var: format!("{loop_name}_iter"),
        name: loop_name,
        id: span,
    }
}

fn validate_num_args(
    expected_num_args: RangeInclusive<usize>,
    actual_num_args: usize,
) -> Result<(), String> {
    if !expected_num_args.contains(&actual_num_args) {
        let (min, max) = (expected_num_args.start(), expected_num_args.end());

        let expected_formatted = format!(
            "{min}{}",
            if min != max {
                format!("-{max}")
            } else {
                String::new()
            }
        );

        Err(format!(
            "expects {expected_formatted} arguments, but got {actual_num_args}"
        ))
    } else {
        Ok(())
    }
}

fn repeat_span(span: Span, count: usize) -> Vec<Span> {
    std::iter::repeat_n(span, count).collect()
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
        Some(Span::new(start, end))
    }
}

#[derive(Debug)]
pub enum CompileError {
    Spanned { span: Span, msg: String },
    Plain(String),
}

impl CompileError {
    pub fn span(&self) -> Option<Span> {
        match self {
            CompileError::Spanned { span, .. } => Some(*span),
            CompileError::Plain(_) => None,
        }
    }

    pub fn msg(&self) -> &str {
        match self {
            CompileError::Spanned { msg, .. } => msg,
            CompileError::Plain(msg) => msg,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct Label(pub usize);
