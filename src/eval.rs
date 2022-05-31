use crate::error::Error;
use std::collections::HashMap;

pub struct Vm<'i, 'c> {
    env: HashMap<&'i str, Slot>,
    stack: Box<[Value; 256]>,
    sp: usize,
    code: &'c [Opcode<'i>],
    ip: usize,
    consts: Vec<Value>,
}

impl<'i, 'c> Vm<'i, 'c> {
    fn new(code: &'c [Opcode<'i>]) -> Self {
        Self {
            env: Default::default(),
            stack: box [const { Value::Undefined }; 256],
            sp: 0,
            code,
            ip: 0,
            consts: Default::default(),
        }
    }

    fn eval(&mut self) -> Result<(), Error> {
        loop {
            let op = &self.code[self.ip];
            println!("{}|\t{:?}", self.ip, op);
            self.ip += 1;

            match op {
                Opcode::Defslot(_, _) => todo!(),
                Opcode::Assign(_) => todo!(),
                Opcode::Read(_) => todo!(),
                &Opcode::Const(c) => self.stack[self.sp] = self.consts[c],
            }

            if self.ip == self.code.len() {
                break;
            }
        }
        Ok(())
    }
}

pub struct Slot {
    pub flags: Flags,
    pub value: Value,
}

bitflags::bitflags! {
    pub struct Flags: u8 {
        const BINDING_MODE_MUT =     0b00000001;
        const BINDING_MODE_IMMUT =   0b00000000;
        const ASSIGNED =             0b00000010;
    }
}

#[derive(Clone, Copy)]
pub enum Value {
    Int(i64),
    Uint(u64),
    Float(f64),
    String(()),
    Undefined,
}

#[derive(Debug)]
pub enum Opcode<'i> {
    Defslot(&'i str, Flags),
    Assign(&'i str),
    Read(&'i str),
    Const(usize),
}
