use crate::{
    compiler::Label,
    grammar::ast::AstValue,
    vm::runtime_value::{function::RuntimeFunction, number::RuntimeNumber, regex::RegexModifiers},
};

#[derive(Debug, Clone)]
pub enum IrValue {
    Null,
    Uninit,
    Bool(bool),
    Int(isize),
    Num(RuntimeNumber),
    Str(String),
    Regex(String, RegexModifiers),
    List(Vec<IrValue>),
    Tuple(Vec<IrValue>),
    Set(Vec<IrValue>),
    Map(Vec<(IrValue, IrValue)>),
    Function(RuntimeFunction<Label>),
}

impl IrValue {
    pub fn new_list() -> Self {
        IrValue::List(Vec::new())
    }

    pub fn new_map() -> Self {
        IrValue::Map(Vec::new())
    }
}

impl<'src> TryFrom<&AstValue<'src>> for IrValue {
    type Error = String;

    fn try_from(val: &AstValue) -> Result<Self, Self::Error> {
        fn collect_try_from(xs: &[AstValue]) -> Result<Vec<IrValue>, String> {
            xs.iter().map(IrValue::try_from).collect()
        }

        let res = match val {
            AstValue::Null => IrValue::Null,
            AstValue::Bool(b) => IrValue::Bool(*b),
            AstValue::Num(n) => IrValue::Num(RuntimeNumber::Float(*n)),
            AstValue::Str(s) => IrValue::Str(s.to_string()),
            AstValue::List(xs) => IrValue::List(collect_try_from(xs)?),
            AstValue::Tuple(xs) => IrValue::Tuple(collect_try_from(xs)?),
            AstValue::Regex(s, modifiers) => IrValue::Regex(s.clone(), modifiers.clone()),
            AstValue::Func(_) => return Err("Functions are not simple values".to_string()),
        };

        Ok(res)
    }
}
