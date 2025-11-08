use oxc_allocator::Allocator;

use crate::{
    compiler::Label,
    grammar::ast::AstValue,
    vm::runtime_value::{
        function::RuntimeFunction,
        number::{FromWithAlloc, RuntimeNumber},
        regex::RegexModifiers,
    },
};

#[derive(Debug, Clone)]
pub enum IrValue {
    Null,
    Uninit,
    Bool(bool),
    Int(isize),
    Num(IrNumber),
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

impl TryFrom<&AstValue<'_>> for IrValue {
    type Error = String;

    fn try_from(val: &AstValue) -> Result<Self, Self::Error> {
        fn collect_try_from(xs: &[AstValue]) -> Result<Vec<IrValue>, String> {
            xs.iter().map(IrValue::try_from).collect()
        }

        let res = match val {
            AstValue::Null => IrValue::Null,
            AstValue::Bool(b) => IrValue::Bool(*b),
            AstValue::Int(n) => IrValue::Num(IrNumber::Int(*n)),
            AstValue::Float(n) => IrValue::Num(IrNumber::Float(*n)),
            AstValue::Str(s) => IrValue::Str(s.to_string()),
            AstValue::List(xs) => IrValue::List(collect_try_from(xs)?),
            AstValue::Tuple(xs) => IrValue::Tuple(collect_try_from(xs)?),
            AstValue::Regex(s, modifiers) => IrValue::Regex(s.clone(), modifiers.clone()),
            AstValue::Func(_) => return Err("Functions are not simple values".to_string()),
        };

        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrNumber {
    Int(i64),
    Float(f64),
}

impl IrNumber {
    pub fn neg(&self) -> Self {
        match self {
            IrNumber::Int(n) => IrNumber::Int(-n),
            IrNumber::Float(n) => IrNumber::Float(-n),
        }
    }

    pub fn to_runtime_number<'gc>(&self, alloc: &'gc Allocator) -> RuntimeNumber<'gc> {
        match self {
            IrNumber::Int(n) => RuntimeNumber::from_with_alloc(*n, alloc),
            IrNumber::Float(n) => RuntimeNumber::Float(*n),
        }
    }
}
