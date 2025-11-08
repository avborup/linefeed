use std::{cell::RefCell, rc::Rc};

use oxc_allocator::{Allocator, Vec as AVec};

use crate::vm::runtime_value::{
    // counter::RuntimeCounter,
    list::RuntimeList,
    map::{MapIterator, RuntimeMap},
    number::RuntimeNumber,
    range::{RangeIterator, RuntimeRange},
    string::RuntimeString,
    tuple::RuntimeTuple,
    RuntimeValue,
};

#[derive(Clone)]
pub struct RuntimeIterator<'gc>(RefCell<IteratorKind<'gc>>);

#[derive(Clone)]
enum IteratorKind<'gc> {
    // List(ListIterator<'gc>),
    Tuple(TupleIterator<'gc>),
    // Range(RangeIterator),
    // Map(MapIterator<'gc>),
    // Counter(CounterIterator),
    Empty,
}

impl<'gc> RuntimeIterator<'gc> {
    pub fn alloc(self, alloc: &'gc Allocator) -> &'gc Self {
        alloc.alloc(self)
    }

    pub fn next(&self) -> Option<RuntimeValue<'gc>> {
        match &mut *self.0.borrow_mut() {
            IteratorKind::Tuple(iter) => iter.next(),
            IteratorKind::Empty => None,
        }
    }

    pub fn to_vec(&self, alloc: &'gc Allocator) -> AVec<'gc, RuntimeValue<'gc>> {
        let mut out = AVec::with_capacity_in(self.len(), alloc);
        while let Some(value) = self.next() {
            out.push(value);
        }
        out
    }

    pub fn len(&self) -> usize {
        match &*self.0.borrow() {
            IteratorKind::Tuple(iter) => iter.tuple.len(),
            IteratorKind::Empty => 0,
        }
    }
}

struct ListIterator<'gc> {
    list: RuntimeList<'gc>,
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
//
// impl<'gc> From<RuntimeList<'gc>> for RuntimeIterator<'gc> {
//     fn from(list: RuntimeList<'gc>) -> Self {
//         Self(Rc::new(RefCell::new(ListIterator { list, index: 0 })))
//     }
// }

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
    list: RuntimeList<'gc>,
    index: usize,
}

impl<'gc> EnumeratedListIterator<'gc> {
    pub fn new(list: RuntimeList<'gc>) -> Self {
        Self { list, index: 0 }
    }
}

// impl<'gc> Iterator for EnumeratedListIterator<'gc> {
//     type Item = RuntimeValue<'gc>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         let value = self.list.as_slice().get(self.index).cloned()?;
//         let index_val = RuntimeValue::Num(RuntimeNumber::from(self.index));
//         let enumerated = todo!(); // RuntimeValue::Tuple(RuntimeTuple::from_vec(vec![index_val, value]));
//         self.index += 1;
//         Some(enumerated)
//     }
// }

impl<'gc> From<EnumeratedListIterator<'gc>> for RuntimeIterator<'gc> {
    fn from(iter: EnumeratedListIterator<'gc>) -> Self {
        todo!()
        // Self(Rc::new(RefCell::new(iter)))
    }
}

// impl<'gc> From<RuntimeRange> for RuntimeIterator<'gc> {
//     fn from(range: RuntimeRange) -> Self {
//         Self(Rc::new(RefCell::new(RangeIterator::new(range))))
//     }
// }

// impl<'gc> From<RuntimeString> for RuntimeIterator<'gc> {
//     fn from(s: RuntimeString) -> Self {
//         Self::from(RuntimeList::from_vec(
//             s.as_str()
//                 .chars()
//                 .map(|c| RuntimeValue::Str(RuntimeString::new(c)))
//                 .collect(),
//         ))
//     }
// }

impl<'gc> From<()> for RuntimeIterator<'gc> {
    fn from(_: ()) -> Self {
        Self(RefCell::new(IteratorKind::Empty))
    }
}

// impl<'gc> From<RuntimeMap<'gc>> for RuntimeIterator<'gc> {
//     fn from(map: RuntimeMap) -> Self {
//         Self(Rc::new(RefCell::new(MapIterator::from(map))))
//     }
// }

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
