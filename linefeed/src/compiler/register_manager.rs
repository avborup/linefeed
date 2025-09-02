use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Debug)]
pub struct RegisterManager {
    max_registers: usize,
    registers: BinaryHeap<Reverse<usize>>,
}

pub const DEFAULT_MAX_REGISTERS: usize = 64;

impl Default for RegisterManager {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_REGISTERS)
    }
}

impl RegisterManager {
    pub fn new(max_registers: usize) -> Self {
        Self {
            max_registers,
            registers: BinaryHeap::from_iter((0..max_registers).map(Reverse)),
        }
    }

    pub fn get_available_register(&mut self) -> Option<usize> {
        self.registers.pop().map(|Reverse(x)| x)
    }

    pub fn free_register(&mut self, register: usize) {
        // This is will be very slow for large numbers of registers, but we don't expect that
        // to ever happen, and we'd like to catch logic errors early.
        if self.registers.iter().any(|Reverse(x)| x == &register) {
            panic!("Register {register} is already free");
        }

        if register < self.max_registers {
            self.registers.push(Reverse(register));
        }
    }
}
