use std::collections::HashMap;

#[derive(Debug)]
pub struct ScopedMap<K, V> {
    scopes: Vec<HashMap<K, V>>,
}

#[derive(Debug, Clone)]
pub enum VarType<T> {
    Local(T),
    Global(T),
    // Upvalue,
}

impl<T> VarType<T> {
    pub fn inner(self) -> T {
        match self {
            Self::Local(val) => val,
            Self::Global(val) => val,
        }
    }
}

impl<K, V> Default for ScopedMap<K, V>
where
    K: Eq + std::hash::Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> ScopedMap<K, V>
where
    K: Eq + std::hash::Hash,
{
    pub fn new() -> Self {
        let mut store = Self { scopes: Vec::new() };
        store.start_scope();
        store
    }

    pub fn start_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
        assert!(!self.scopes.is_empty(), "Cannot pop the last scope");
    }

    pub fn get(&self, key: &K) -> Option<VarType<&V>> {
        let cur_scope = self.scopes.len() - 1;
        self.scopes.iter().enumerate().rev().find_map(|(i, scope)| {
            scope.get(key).and_then(|v| match i {
                0 => Some(VarType::Global(v)),
                _ if i == cur_scope => Some(VarType::Local(v)),
                _ => None,
            })
        })
    }

    pub fn get_mut(&mut self, key: &K) -> Option<VarType<&mut V>> {
        let cur_scope = self.scopes.len() - 1;
        self.scopes
            .iter_mut()
            .enumerate()
            .rev()
            .find_map(|(i, scope)| {
                scope.get_mut(key).and_then(|v| match i {
                    0 => Some(VarType::Global(v)),
                    _ if i == cur_scope => Some(VarType::Local(v)),
                    _ => None,
                })
            })
    }

    pub fn set(&mut self, key: K, val: V) {
        match self.get_mut(&key) {
            Some(existing) => *existing.inner() = val,
            None => self.set_local(key, val),
        }
    }

    pub fn get_local(&self, name: &K) -> Option<&V> {
        self.scopes.last().unwrap().get(name)
    }

    pub fn set_local(&mut self, name: K, val: V) {
        self.scopes.last_mut().unwrap().insert(name, val);
    }

    pub fn cur_scope_len(&self) -> usize {
        self.scopes.last().unwrap().len()
    }

    pub fn is_currently_top_scope(&self) -> bool {
        self.scopes.len() == 1
    }
}
