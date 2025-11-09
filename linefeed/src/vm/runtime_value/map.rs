use std::{cell::RefCell, rc::Rc};

use ouroboros::self_referencing;
use rustc_hash::FxHashMap;

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
    pub map: FxHashMap<RuntimeValue, RuntimeValue>,
    pub default_value: Option<RuntimeValue>,
}

impl RuntimeMap {
    pub fn new() -> Self {
        Self::from_map(FxHashMap::default())
    }

    pub fn from_map(map: FxHashMap<RuntimeValue, RuntimeValue>) -> Self {
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
        let new_map = Self::from_map(
            self.borrow()
                .iter()
                .map(|(k, v)| (k.deep_clone(), v.deep_clone()))
                .collect(),
        );

        if let Some(default_value) = &self.borrow().default_value {
            new_map.borrow_mut().default_value = Some(default_value.deep_clone());
        }

        new_map
    }

    pub fn get(&self, key: &RuntimeValue) -> RuntimeValue {
        self.insert_default_value_if_missing(key);

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

    fn insert_default_value_if_missing(&self, key: &RuntimeValue) {
        let Some(default_value) = self.borrow().default_value.clone() else {
            return;
        };

        if !self.borrow().contains_key(key) {
            self.insert(key.clone(), default_value.deep_clone());
        }
    }
}

impl std::ops::Deref for InnerRuntimeMap {
    type Target = FxHashMap<RuntimeValue, RuntimeValue>;

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

        a.len() == b.len() && a.iter().all(|(key, val)| b.get(key) == Some(val))
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
        let mut map = FxHashMap::default();
        while let Some(item) = iter.next() {
            let key = item.index(&RuntimeValue::Num(RuntimeNumber::from(0)))?;
            let val = item.index(&RuntimeValue::Num(RuntimeNumber::from(1)))?;
            map.insert(key, val);
        }
        Ok(Self::from_map(map))
    }
}

#[self_referencing]
struct MapIterCell {
    owner: RuntimeMap,
    #[borrows(owner)]
    #[covariant]
    guard: std::cell::Ref<'this, InnerRuntimeMap>,
    #[borrows(guard)]
    #[covariant]
    iter: std::collections::hash_map::Iter<'this, RuntimeValue, RuntimeValue>,
}

pub struct MapIterator {
    cell: MapIterCell,
    len: usize,
}

impl MapIterator {
    fn new(map: RuntimeMap) -> Self {
        let len = map.borrow().len();
        let cell = MapIterCellBuilder {
            owner: map,
            guard_builder: |owner| owner.borrow(),
            iter_builder: |guard| guard.iter(),
        }
        .build();

        Self { cell, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl From<RuntimeMap> for MapIterator {
    fn from(map: RuntimeMap) -> Self {
        Self::new(map)
    }
}

impl Iterator for MapIterator {
    type Item = RuntimeValue;

    fn next(&mut self) -> Option<Self::Item> {
        self.cell
            .with_iter_mut(|it| it.next())
            .map(|(k, v)| RuntimeValue::Tuple(RuntimeTuple::from_vec(vec![k.clone(), v.clone()])))
    }
}
