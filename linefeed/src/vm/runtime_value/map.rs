use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::vm::{
    runtime_value::{
        iterator::RuntimeIterator, number::RuntimeNumber, tuple::RuntimeTuple, RuntimeValue,
    },
    RuntimeError,
};

#[derive(Debug, Clone)]
pub struct RuntimeMap(Rc<RefCell<InnerRuntimeMap>>);

#[derive(Debug, Clone)]
pub struct InnerRuntimeMap {
    pub map: HashMap<RuntimeValue, RuntimeValue>,
    pub default_value: Option<RuntimeValue>,
}

impl RuntimeMap {
    pub fn new() -> Self {
        Self::from_map(HashMap::new())
    }

    pub fn from_map(map: HashMap<RuntimeValue, RuntimeValue>) -> Self {
        Self(Rc::new(RefCell::new(InnerRuntimeMap {
            map,
            default_value: None,
        })))
    }

    pub fn new_with_default_value(default_value: RuntimeValue) -> Self {
        let runtime_map = Self::new();
        runtime_map.borrow_mut().default_value = Some(default_value);
        runtime_map
    }

    pub fn len(&self) -> usize {
        self.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, InnerRuntimeMap> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, InnerRuntimeMap> {
        self.0.borrow_mut()
    }

    pub fn deep_clone(&self) -> Self {
        Self::from_map(
            self.borrow()
                .iter()
                .map(|(k, v)| (k.deep_clone(), v.deep_clone()))
                .collect(),
        )
    }

    pub fn get(&self, key: &RuntimeValue) -> RuntimeValue {
        self.borrow()
            .get(key)
            .cloned()
            .unwrap_or(RuntimeValue::Null)
    }

    pub fn insert(&self, key: RuntimeValue, value: RuntimeValue) {
        self.borrow_mut().insert(key, value);
    }

    pub fn contains_key(&self, key: &RuntimeValue) -> bool {
        self.borrow().contains_key(key)
    }
}

impl std::ops::Deref for InnerRuntimeMap {
    type Target = HashMap<RuntimeValue, RuntimeValue>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl std::ops::DerefMut for InnerRuntimeMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl Default for RuntimeMap {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for RuntimeMap {
    fn eq(&self, other: &Self) -> bool {
        let a = self.borrow();
        let b = other.borrow();

        a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| a == b)
    }
}

impl Eq for RuntimeMap {}

impl std::hash::Hash for RuntimeMap {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let set = self.borrow();
        let mut items = set.iter().collect::<Vec<_>>();
        items.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        items.hash(state);
    }
}

impl std::cmp::PartialOrd for RuntimeMap {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.borrow();
        let b = other.borrow();
        a.len().partial_cmp(&b.len())
    }
}

impl TryFrom<RuntimeIterator> for RuntimeMap {
    type Error = RuntimeError;

    fn try_from(iter: RuntimeIterator) -> Result<Self, Self::Error> {
        let mut map = HashMap::new();
        while let Some(item) = iter.next() {
            let key = item.index(&RuntimeValue::Num(RuntimeNumber::from(0)))?;
            let val = item.index(&RuntimeValue::Num(RuntimeNumber::from(1)))?;
            map.insert(key, val);
        }
        Ok(Self::from_map(map))
    }
}

pub struct MapIterator {
    map: RuntimeMap,
    keys: Vec<RuntimeValue>,
    index: usize,
}

impl From<RuntimeMap> for MapIterator {
    fn from(map: RuntimeMap) -> Self {
        let keys = map.borrow().keys().cloned().collect();
        Self {
            map,
            keys,
            index: 0,
        }
    }
}

impl Iterator for MapIterator {
    type Item = RuntimeValue;

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.keys.get(self.index).cloned()?;
        let value = self.map.borrow().get(&key).cloned()?;
        let pair = RuntimeValue::Tuple(RuntimeTuple::from_vec(vec![key, value]));

        self.index += 1;

        Some(pair)
    }
}
