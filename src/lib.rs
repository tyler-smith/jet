#![feature(allocator_api)]

use crate::instructions::Instruction;

pub mod builder;
pub mod engine;
pub mod instructions;
pub mod runtime;

pub struct ROMIterator<'a> {
    pc: usize,
    rom: &'a [u8],
}

impl<'a> ROMIterator<'a> {
    pub fn new(rom: &'a [u8]) -> Self {
        Self {
            pc: 0,
            rom,
        }
    }
}

pub enum ROMIteratorItem<'a> {
    Instr(usize, Instruction),
    PushData(usize, &'a [u8]),
    Invalid(usize),
}

impl<'a> Iterator for ROMIterator<'a> {
    type Item = ROMIteratorItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Stop iterating if we're at the end of the ROM
        if self.pc >= self.rom.len() {
            return None;
        }

        // If the next byte isn't a valid instruction, return an error
        let current_byte = self.rom[self.pc];
        let instr = Instruction::try_from(current_byte);
        let instr = if let Ok(instr) = instr {
            instr
        } else {
            return Some(ROMIteratorItem::Invalid(self.pc));
        };

        // If the instruction is not a PUSH then increment the PC and return the instruction
        if !instr.is_push() {
            let pc = self.pc;
            self.pc += 1;
            return Some(ROMIteratorItem::Instr(pc, instr));
        };

        // We have a PUSH instruction, so emit the next N bytes
        let push_len = instr as usize - Instruction::PUSH1 as usize + 1;
        let data = &self.rom[self.pc + 1..self.pc + 1 + push_len];
        let pc = self.pc;
        self.pc += push_len + 1;
        Some(ROMIteratorItem::PushData(pc, data))
    }
}
