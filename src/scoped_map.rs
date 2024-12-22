use std::collections::HashMap;

#[derive(Debug)]
pub struct ScopedMap<K, V> {
    scopes: Vec<HashMap<K, V>>,
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
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.scopes.iter().rev().find_map(|scope| scope.get(key))
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(key))
    }

    pub fn set(&mut self, key: K, val: V) {
        match self.get_mut(&key) {
            Some(existing) => *existing = val,
            None => self.set_local(key, val),
        }
    }

    pub fn set_local(&mut self, name: K, val: V) {
        self.scopes.last_mut().unwrap().insert(name, val);
    }
}
