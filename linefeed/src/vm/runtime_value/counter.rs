use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::vm::{
    runtime_value::{
        iterator::RuntimeIterator,
        map::{MapIterator, RuntimeMap},
        number::RuntimeNumber,
        RuntimeValue,
    },
    RuntimeError,
};

#[derive(Debug, Clone)]
pub struct RuntimeCounter(Rc<RefCell<InnerRuntimeCounter>>);

#[derive(Debug, Clone)]
pub struct InnerRuntimeCounter {
    pub map: HashMap<RuntimeValue, isize>,
}

impl RuntimeCounter {
    pub fn new() -> Self {
        Self::from_map(HashMap::new())
    }

    pub fn from_map(map: HashMap<RuntimeValue, isize>) -> Self {
        Self(Rc::new(RefCell::new(InnerRuntimeCounter { map })))
    }

    pub fn len(&self) -> usize {
        self.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, InnerRuntimeCounter> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, InnerRuntimeCounter> {
        self.0.borrow_mut()
    }

    pub fn into_runtime_map(&self) -> RuntimeMap {
        let map = self
            .borrow()
            .iter()
            .map(|(k, v)| (k.deep_clone(), RuntimeValue::Num(RuntimeNumber::from(*v))))
            .collect();

        RuntimeMap::from_map(map)
    }

    pub fn deep_clone(&self) -> Self {
        let new_map = Self::from_map(
            self.borrow()
                .iter()
                .map(|(k, v)| (k.deep_clone(), *v))
                .collect(),
        );

        new_map
    }

    pub fn get(&self, key: &RuntimeValue) -> RuntimeValue {
        let count = self.borrow().get(key).map(|c| *c).unwrap_or(0);
        RuntimeValue::Num(RuntimeNumber::from(count))
    }

    pub fn add(&self, key: RuntimeValue, amount: isize) {
        self.borrow_mut()
            .entry(key)
            .and_modify(|v| *v += amount)
            .or_insert_with(|| amount);
    }

    pub fn sub(&self, key: &RuntimeValue, amount: isize) {
        self.borrow_mut()
            .entry(key.clone())
            .and_modify(|v| *v -= amount)
            .or_insert_with(|| -amount);
    }

    pub fn contains_key(&self, key: &RuntimeValue) -> bool {
        self.borrow().contains_key(key)
    }
}

impl std::ops::Deref for InnerRuntimeCounter {
    type Target = HashMap<RuntimeValue, isize>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl std::ops::DerefMut for InnerRuntimeCounter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl Default for RuntimeCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for RuntimeCounter {
    fn eq(&self, other: &Self) -> bool {
        let a = self.borrow();
        let b = other.borrow();

        a.len() == b.len() && a.iter().all(|(key, val)| b.get(key) == Some(val))
    }
}

impl Eq for RuntimeCounter {}

impl std::hash::Hash for RuntimeCounter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let set = self.borrow();
        let mut items = set.iter().collect::<Vec<_>>();
        items.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        items.hash(state);
    }
}

impl std::cmp::PartialOrd for RuntimeCounter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.borrow();
        let b = other.borrow();
        a.len().partial_cmp(&b.len())
    }
}

impl TryFrom<RuntimeIterator> for RuntimeCounter {
    type Error = RuntimeError;

    fn try_from(iter: RuntimeIterator) -> Result<Self, Self::Error> {
        let counter = Self::new();
        while let Some(item) = iter.next() {
            counter.add(item, 1);
        }
        Ok(counter)
    }
}

impl From<RuntimeCounter> for MapIterator {
    fn from(counter: RuntimeCounter) -> Self {
        MapIterator::from(counter.into_runtime_map())
    }
}
