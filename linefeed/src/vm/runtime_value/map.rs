use std::{cell::RefCell, mem::ManuallyDrop, ops::Deref};

use oxc_allocator::{Allocator, Box as ABox, CloneIn, HashMap as AHashMap, Vec as AVec};

use crate::vm::runtime_value::{tuple::RuntimeTuple, RuntimeValue};

#[derive(Debug)]
pub struct RuntimeMap<'gc>(RefCell<InnerRuntimeMap<'gc>>);

#[derive(Debug)]
pub struct InnerRuntimeMap<'gc> {
    pub map: AHashMap<'gc, RuntimeValue<'gc>, RuntimeValue<'gc>>,
    pub default_value: Option<ABox<'gc, RuntimeValue<'gc>>>,
}

impl<'gc> RuntimeMap<'gc> {
    pub fn from_iter(
        iter: impl IntoIterator<Item = (RuntimeValue<'gc>, RuntimeValue<'gc>)>,
        alloc: &'gc Allocator,
    ) -> Self {
        Self::from_map(AHashMap::from_iter_in(iter, alloc))
    }

    pub fn from_map(map: AHashMap<'gc, RuntimeValue<'gc>, RuntimeValue<'gc>>) -> Self {
        Self(RefCell::new(InnerRuntimeMap {
            map,
            default_value: None,
        }))
    }

    pub fn new_with_default_value(default_value: RuntimeValue<'gc>, alloc: &'gc Allocator) -> Self {
        let runtime_map = Self::from_map(AHashMap::new_in(alloc));
        runtime_map.borrow_mut().default_value = Some(ABox::new_in(default_value, alloc));
        runtime_map
    }

    pub fn alloc(self, alloc: &'gc Allocator) -> &'gc Self {
        alloc.alloc(self)
    }

    pub fn len(&self) -> usize {
        self.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, InnerRuntimeMap<'gc>> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, InnerRuntimeMap<'gc>> {
        self.0.borrow_mut()
    }

    pub fn get(&self, key: &RuntimeValue<'gc>, alloc: &'gc Allocator) -> RuntimeValue<'gc> {
        self.insert_default_value_if_missing(key, alloc);

        self.borrow()
            .get(key)
            .cloned()
            .unwrap_or(RuntimeValue::Null)
    }

    pub fn insert(&self, key: RuntimeValue<'gc>, value: RuntimeValue<'gc>) {
        self.borrow_mut().insert(key, value);
    }

    pub fn contains_key(&self, key: &RuntimeValue<'gc>) -> bool {
        self.borrow().contains_key(key)
    }

    fn insert_default_value_if_missing(&self, key: &RuntimeValue<'gc>, alloc: &'gc Allocator) {
        let to_insert = {
            let inner = self.0.borrow();

            let Some(default_box) = inner.default_value.as_ref() else {
                return;
            };

            if inner.map.contains_key(key) {
                return;
            }

            default_box.as_ref().clone_in(alloc)
        };

        let mut inner = self.0.borrow_mut();

        inner.map.insert(key.clone(), to_insert);
    }
}

impl<'old, 'new> oxc_allocator::CloneIn<'new> for RuntimeMap<'old> {
    type Cloned = RuntimeMap<'new>;

    fn clone_in(&self, alloc: &'new Allocator) -> Self::Cloned {
        let cloned = self.0.borrow().iter().fold(
            AHashMap::with_capacity_in(self.len(), alloc),
            |mut acc, (k, v)| {
                let k = k.clone_in(alloc);
                let v = v.clone_in(alloc);
                acc.insert(k, v);
                acc
            },
        );

        RuntimeMap::from_map(cloned)
    }
}

impl<'gc> std::ops::Deref for InnerRuntimeMap<'gc> {
    type Target = AHashMap<'gc, RuntimeValue<'gc>, RuntimeValue<'gc>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl std::ops::DerefMut for InnerRuntimeMap<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl PartialEq for RuntimeMap<'_> {
    fn eq(&self, other: &Self) -> bool {
        let a = self.borrow();
        let b = other.borrow();

        a.len() == b.len() && a.iter().all(|(key, val)| b.get(key) == Some(val))
    }
}

impl Eq for RuntimeMap<'_> {}

impl std::hash::Hash for RuntimeMap<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let set = self.borrow();
        let mut items = set.iter().collect::<Vec<_>>();
        items.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        items.hash(state);
    }
}

impl std::cmp::PartialOrd for RuntimeMap<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.borrow();
        let b = other.borrow();
        a.len().partial_cmp(&b.len())
    }
}

// impl TryFrom<RuntimeIterator> for RuntimeMap<'_> {
//     type Error = RuntimeError;
//
//     fn try_from(iter: RuntimeIterator) -> Result<Self, Self::Error> {
//         let mut map = FxHashMap::default();
//         while let Some(item) = iter.next() {
//             let key = item.index(&RuntimeValue::Num(RuntimeNumber::from(0)))?;
//             let val = item.index(&RuntimeValue::Num(RuntimeNumber::from(1)))?;
//             map.insert(key, val);
//         }
//         Ok(Self::from_map(map))
//     }
// }

use ouroboros::self_referencing;
use oxc_allocator::hash_map::Iter as HbIter;
use std::cell::Ref;

#[self_referencing]
struct MapIterCell<'gc> {
    owner: Ref<'gc, InnerRuntimeMap<'gc>>,
    #[borrows(owner)]
    #[covariant]
    iter: HbIter<'this, RuntimeValue<'gc>, RuntimeValue<'gc>>,
}

pub struct MapIterator<'gc> {
    cell: ManuallyDrop<MapIterCell<'gc>>,
    len: usize,
}

impl<'gc> MapIterator<'gc> {
    pub fn new(map: &'gc RuntimeMap<'gc>) -> Self {
        let len = map.borrow().len(); // short borrow just to read len
        let cell = MapIterCell::new(map.borrow(), |guard| guard.iter());
        Self {
            cell: ManuallyDrop::new(cell),
            len,
        }
    }

    pub fn next(&mut self, alloc: &'gc Allocator) -> Option<RuntimeValue<'gc>> {
        self.cell.with_iter_mut(|it| it.next()).map(|(k, v)| {
            let pair_vec = AVec::from_iter_in([k.clone(), v.clone()], alloc);
            RuntimeValue::Tuple(RuntimeTuple::from_vec(pair_vec).alloc(alloc))
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
