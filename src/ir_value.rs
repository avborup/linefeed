use regex::Regex;

use crate::{
    ast::AstValue,
    compiler::Label,
    runtime_value::{function::RuntimeFunction, number::RuntimeNumber},
};

#[derive(Debug, Clone)]
pub enum IrValue {
    Null,
    Uninit,
    Bool(bool),
    Int(isize),
    Num(RuntimeNumber),
    Str(String),
    Regex(Regex),
    List(IrList),
    Set(IrList),
    Function(RuntimeFunction<Label>),
}

#[derive(Debug, Clone)]
pub struct IrList(pub Vec<IrValue>);

impl TryFrom<&AstValue> for IrValue {
    type Error = String;

    fn try_from(val: &AstValue) -> Result<Self, Self::Error> {
        let res = match val {
            AstValue::Null => IrValue::Null,
            AstValue::Bool(b) => IrValue::Bool(*b),
            AstValue::Num(n) => IrValue::Num(RuntimeNumber::Float(*n)),
            AstValue::Str(s) => IrValue::Str(s.clone()),
            AstValue::List(xs) => {
                let items = xs.iter().map(IrValue::try_from).collect::<Result<_, _>>()?;
                IrValue::List(IrList(items))
            }
            AstValue::Func(_) => return Err("Functions are not simple values".to_string()),
            AstValue::Regex(s) => Regex::new(s)
                .map(IrValue::Regex)
                .map_err(|e| format!("Invalid regex: {}", e))?,
        };

        Ok(res)
    }
}
