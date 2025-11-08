use std::{cell::RefCell, rc::Rc};

use oxc_allocator::{Allocator, Vec as AVec};

use crate::vm::runtime_value::{
    list::RuntimeList,
    map::{MapIterator, RuntimeMap},
    number::RuntimeNumber,
    range::{RangeIterator, RuntimeRange},
    string::{RuntimeString, StringIterator},
    tuple::RuntimeTuple,
    // counter::RuntimeCounter,
    RuntimeValue,
};

pub struct RuntimeIterator<'gc>(RefCell<IteratorKind<'gc>>);

enum IteratorKind<'gc> {
    List(ListIterator<'gc>),
    Tuple(TupleIterator<'gc>),
    Range(RangeIterator<'gc>),
    Map(MapIterator<'gc>),
    Enumerated(EnumeratedListIterator<'gc>),
    String(StringIterator<'gc>),
    // Counter(CounterIterator),
    Empty,
}

impl<'gc> RuntimeIterator<'gc> {
    pub fn alloc(self, alloc: &'gc Allocator) -> &'gc Self {
        alloc.alloc(self)
    }

    pub fn next(&self, alloc: &'gc Allocator) -> Option<RuntimeValue<'gc>> {
        match &mut *self.0.borrow_mut() {
            IteratorKind::List(iter) => iter.next(),
            IteratorKind::Tuple(iter) => iter.next(),
            IteratorKind::Range(iter) => iter.next(),
            IteratorKind::Map(iter) => iter.next(alloc),
            IteratorKind::Enumerated(iter) => iter.next(alloc),
            IteratorKind::String(iter) => iter.next(),
            IteratorKind::Empty => None,
        }
    }

    pub fn to_vec(&self, alloc: &'gc Allocator) -> AVec<'gc, RuntimeValue<'gc>> {
        let mut out = AVec::with_capacity_in(self.len(), alloc);
        while let Some(value) = self.next(alloc) {
            out.push(value);
        }
        out
    }

    pub fn len(&self) -> usize {
        match &*self.0.borrow() {
            IteratorKind::List(iter) => iter.list.len(),
            IteratorKind::Tuple(iter) => iter.tuple.len(),
            IteratorKind::Range(iter) => iter.len().unwrap_or(usize::MAX),
            IteratorKind::Map(iter) => iter.len(),
            IteratorKind::Enumerated(iter) => iter.list.len(),
            IteratorKind::String(iter) => iter.chars.len(),
            IteratorKind::Empty => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

struct ListIterator<'gc> {
    list: &'gc RuntimeList<'gc>,
    index: usize,
}

impl<'gc> Iterator for ListIterator<'gc> {
    type Item = RuntimeValue<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.list.as_slice().get(self.index).cloned();
        self.index += 1;
        value
    }
}

impl<'gc> From<&'gc RuntimeList<'gc>> for RuntimeIterator<'gc> {
    fn from(list: &'gc RuntimeList<'gc>) -> Self {
        Self(RefCell::new(IteratorKind::List(ListIterator {
            list,
            index: 0,
        })))
    }
}

#[derive(Clone)]
struct TupleIterator<'gc> {
    tuple: &'gc RuntimeTuple<'gc>,
    index: usize,
}

impl<'gc> TupleIterator<'gc> {
    fn next(&mut self) -> Option<RuntimeValue<'gc>> {
        let value = self.tuple.as_slice().get(self.index).cloned();
        self.index += 1;
        value
    }
}

impl<'gc> From<&'gc RuntimeTuple<'gc>> for RuntimeIterator<'gc> {
    fn from(tuple: &'gc RuntimeTuple<'gc>) -> Self {
        Self(RefCell::new(IteratorKind::Tuple(TupleIterator {
            tuple,
            index: 0,
        })))
    }
}

pub struct EnumeratedListIterator<'gc> {
    list: &'gc RuntimeList<'gc>,
    index: isize,
}

impl<'gc> EnumeratedListIterator<'gc> {
    pub fn new(list: &'gc RuntimeList<'gc>) -> Self {
        Self { list, index: 0 }
    }

    fn next(&mut self, alloc: &'gc Allocator) -> Option<RuntimeValue<'gc>> {
        let value = self.list.as_slice().get(self.index as usize).cloned()?;
        let index_val = RuntimeValue::Num(RuntimeNumber::from(self.index));

        let pair_vec = AVec::from_iter_in([index_val, value], alloc);
        let enumerated = RuntimeValue::Tuple(RuntimeTuple::from_vec(pair_vec).alloc(alloc));

        self.index += 1;

        Some(enumerated)
    }
}

impl<'gc> From<EnumeratedListIterator<'gc>> for RuntimeIterator<'gc> {
    fn from(iter: EnumeratedListIterator<'gc>) -> Self {
        Self(RefCell::new(IteratorKind::Enumerated(iter)))
    }
}

impl<'gc> From<RuntimeRange> for RuntimeIterator<'gc> {
    fn from(range: RuntimeRange) -> Self {
        Self(RefCell::new(IteratorKind::Range(RangeIterator::new(range))))
    }
}

impl<'gc> From<StringIterator<'gc>> for RuntimeIterator<'gc> {
    fn from(s: StringIterator<'gc>) -> Self {
        Self(RefCell::new(IteratorKind::String(s)))
    }
}

impl<'gc> From<()> for RuntimeIterator<'gc> {
    fn from(_: ()) -> Self {
        Self(RefCell::new(IteratorKind::Empty))
    }
}

impl<'gc> From<MapIterator<'gc>> for RuntimeIterator<'gc> {
    fn from(map_iter: MapIterator<'gc>) -> Self {
        Self(RefCell::new(IteratorKind::Map(MapIterator::from(map_iter))))
    }
}

// impl From<RuntimeCounter> for RuntimeIterator {
//     fn from(map: RuntimeCounter) -> Self {
//         Self(Rc::new(RefCell::new(MapIterator::from(map))))
//     }
// }

impl std::cmp::PartialEq for RuntimeIterator<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl std::cmp::Eq for RuntimeIterator<'_> {}

impl std::fmt::Debug for RuntimeIterator<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RuntimeIterator({:p})", self.0.as_ptr())
    }
}

impl std::fmt::Display for RuntimeIterator<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<iterator@{:?}>", self.0.as_ptr())
    }
}

impl std::hash::Hash for RuntimeIterator<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state)
    }
}
