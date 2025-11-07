use std::{cell::RefCell, rc::Rc};

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
pub struct RuntimeIterator<'gc>(Rc<RefCell<dyn Iterator<Item = RuntimeValue<'gc>>>>);

impl<'gc> RuntimeIterator<'gc> {
    pub fn next(&self) -> Option<RuntimeValue<'gc>> {
        self.0.borrow_mut().next()
    }

    pub fn to_vec(&self) -> Vec<RuntimeValue<'gc>> {
        let mut out = Vec::new();
        while let Some(value) = self.next() {
            out.push(value);
        }
        out
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

struct TupleIterator<'gc> {
    tuple: RuntimeTuple<'gc>,
    index: usize,
}

impl<'gc> Iterator for TupleIterator<'gc> {
    type Item = RuntimeValue<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.tuple.as_slice().get(self.index).cloned();
        self.index += 1;
        value
    }
}

// impl<'gc> From<RuntimeTuple<'gc>> for RuntimeIterator<'gc> {
//     fn from(tuple: RuntimeTuple) -> Self {
//         Self(Rc::new(RefCell::new(TupleIterator { tuple, index: 0 })))
//     }
// }

pub struct EnumeratedListIterator<'gc> {
    list: RuntimeList<'gc>,
    index: usize,
}

impl<'gc> EnumeratedListIterator<'gc> {
    pub fn new(list: RuntimeList<'gc>) -> Self {
        Self { list, index: 0 }
    }
}

impl<'gc> Iterator for EnumeratedListIterator<'gc> {
    type Item = RuntimeValue<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.list.as_slice().get(self.index).cloned()?;
        let index_val = RuntimeValue::Num(RuntimeNumber::from(self.index));
        let enumerated = todo!(); // RuntimeValue::Tuple(RuntimeTuple::from_vec(vec![index_val, value]));
        self.index += 1;
        Some(enumerated)
    }
}

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

pub struct EmptyIterator;

impl Iterator for EmptyIterator {
    type Item = RuntimeValue<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl From<()> for RuntimeIterator<'static> {
    fn from(_: ()) -> Self {
        Self(Rc::new(RefCell::new(EmptyIterator)))
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
        Rc::ptr_eq(&self.0, &other.0)
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
