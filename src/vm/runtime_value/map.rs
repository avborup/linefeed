use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::vm::runtime_value::{tuple::RuntimeTuple, RuntimeValue};

#[derive(Debug, Clone)]
pub struct RuntimeMap(Rc<RefCell<HashMap<RuntimeValue, RuntimeValue>>>);

impl RuntimeMap {
    pub fn new() -> Self {
        Self::from_map(HashMap::new())
    }

    pub fn from_map(map: HashMap<RuntimeValue, RuntimeValue>) -> Self {
        Self(Rc::new(RefCell::new(map)))
    }

    pub fn len(&self) -> usize {
        self.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, HashMap<RuntimeValue, RuntimeValue>> {
        self.0.borrow()
    }

    pub fn deep_clone(&self) -> Self {
        Self::from_map(
            self.0
                .borrow()
                .iter()
                .map(|(k, v)| (k.deep_clone(), v.deep_clone()))
                .collect(),
        )
    }

    pub fn get(&self, key: &RuntimeValue) -> RuntimeValue {
        self.0
            .borrow()
            .get(key)
            .cloned()
            .unwrap_or(RuntimeValue::Null)
    }

    pub fn insert(&self, key: RuntimeValue, value: RuntimeValue) {
        self.0.borrow_mut().insert(key, value);
    }

    pub fn contains_key(&self, key: &RuntimeValue) -> bool {
        self.0.borrow().contains_key(key)
    }
}

impl Default for RuntimeMap {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for RuntimeMap {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0.borrow();
        let b = other.0.borrow();

        a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| a == b)
    }
}

impl Eq for RuntimeMap {}

impl std::hash::Hash for RuntimeMap {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let set = self.0.borrow();
        let mut items = set.iter().collect::<Vec<_>>();
        items.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        items.hash(state);
    }
}

impl std::cmp::PartialOrd for RuntimeMap {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.0.borrow();
        let b = other.0.borrow();
        a.len().partial_cmp(&b.len())
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
