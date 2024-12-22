// TODO: Make all arguments generic/polymorphic, generate code for all possible types. Type inference.

use std::rc::Rc;

use crate::{
    ast::{Expr, Span, Spanned, UnaryOp, Value as AstValue},
    scoped_map::ScopedMap,
};

#[derive(Debug, Clone)]
pub enum Instruction {
    Load,
    Store,
    Address(usize),
    PrintValue,
    Value(RuntimeValue),
    GetBasePtr,
    Add,
    Sub,
    Mul,
    Div,
    Constant(isize),
    Not,
}

use Instruction::*;

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Null,
    Bool(bool),
    Num(f64),
    Str(Rc<String>),
    List(Rc<Vec<RuntimeValue>>),
}

#[derive(Default)]
pub struct Compiler {
    vars: ScopedMap<String, usize>,
}

impl Compiler {
    pub fn compile_expr(&mut self, expr: &Spanned<Expr>) -> Result<Vec<Instruction>, CompileError> {
        let instructions = match &expr.0 {
            Expr::Error => unreachable!(),

            Expr::Local(name) => {
                self.compile_var_access(name)
                    .map_err(|msg| CompileError::Spanned {
                        span: expr.1.clone(),
                        msg: format!("Failed to compile variable access to '{name}': {msg}"),
                    })?
            }

            Expr::Let(name, val) => self.compile_var_assign(name, val)?,

            Expr::Value(val) => self
                .compile_value(val)
                .map_err(|msg| CompileError::Spanned {
                    span: expr.1.clone(),
                    msg,
                })?,

            Expr::Sequence(exprs) => exprs
                .iter()
                .map(|expr| self.compile_expr(expr))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect(),

            Expr::Print(expr) => {
                let mut instrs = self.compile_expr(expr)?;
                instrs.push(PrintValue);
                instrs
            }

            Expr::Block(expr) => self.compile_expr(expr)?,

            Expr::Unary(op, expr) => {
                let mut instrs = self.compile_expr(expr)?;

                match op {
                    UnaryOp::Not => instrs.push(Not),
                    UnaryOp::Neg => instrs.extend([Constant(-1), Mul]),
                }

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

        Ok(vec![GetBasePtr, Address(*addr), Add, Load])
    }

    // FIXME: Same as above
    fn compile_var_assign(
        &mut self,
        name: &String,
        val: &Spanned<Expr>,
    ) -> Result<Vec<Instruction>, CompileError> {
        let addr = self.vars.get(name).cloned().unwrap_or_else(|| {
            let local_addr = self.vars.cur_scope_len();
            self.vars.set(name.clone(), local_addr);
            local_addr
        });

        let mut store_instrs = vec![GetBasePtr, Address(addr), Add];
        store_instrs.extend(self.compile_expr(val)?);
        store_instrs.push(Store);

        Ok(store_instrs)
    }

    fn compile_value(&mut self, val: &AstValue) -> Result<Vec<Instruction>, String> {
        let rt_val = RuntimeValue::try_from(val)?;
        Ok(vec![Instruction::Value(rt_val)])
    }
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
