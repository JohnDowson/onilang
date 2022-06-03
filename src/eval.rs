use crate::{compiler::FuncProto, error::Error};
use lasso::{Rodeo, Spur};
use std::{collections::HashMap, rc::Rc, sync::RwLock};

pub struct Env {
    parent: Option<Rc<Env>>,
    inner: RwLock<HashMap<Spur, Slot>>,
}

impl Env {
    fn new(interner: &mut Rodeo) -> Self {
        let mut env = HashMap::default();
        env.insert(
            interner.get_or_intern_static("print"),
            Slot {
                flags: Flags::ASSIGNED,
                value: Value::Func(RuntimeFunc::Native(|vm| {
                    let arg = vm.stack.pop().unwrap();
                    println!("{:?}", arg);
                })),
            },
        );
        Self {
            parent: None,
            inner: RwLock::new(env),
        }
    }

    fn with_parent(parent: Rc<Env>) -> Self {
        Self {
            parent: Some(parent),
            inner: RwLock::new(HashMap::new()),
        }
    }

    fn defslot(&self, k: Spur, flags: Flags) -> Result<Option<Slot>, Error> {
        Ok(self
            .inner
            .write()
            .map_err(|e| Error::eval(concat!(file!(), ":", line!())))?
            .insert(
                k,
                Slot {
                    flags,
                    value: Value::Undefined,
                },
            ))
    }

    fn assign(&self, k: &Spur, v: Value) -> Result<(), Error> {
        if let Some(slot) = self
            .inner
            .write()
            .map_err(|e| Error::eval(concat!(file!(), ":", line!())))?
            .get_mut(k)
        {
            slot.flags |= Flags::ASSIGNED;
            slot.value = v;
            Ok(())
        } else {
            Err(Error::eval(concat!(file!(), ":", line!())))
        }
    }

    fn read(&self, k: &Spur) -> Result<Value, Error> {
        self.inner
            .read()
            .map_err(|e| Error::eval(concat!(file!(), ":", line!())))?
            .get(k)
            .map(|s| s.value.clone())
            .ok_or_else(|| Error::eval(concat!(file!(), ":", line!())))
    }
}

pub struct Vm<'i> {
    env: Env,
    interner: &'i mut Rodeo,
    stack: Vec<Value>,
    code: Rc<[Opcode]>,
    ip: usize,
    consts: Vec<ConstValue>,
}

impl<'i> Vm<'i> {
    pub fn new(code: Rc<[Opcode]>, consts: Vec<ConstValue>, interner: &'i mut Rodeo) -> Self {
        Self {
            env: Env::new(interner),
            interner,
            stack: Vec::new(),
            code,
            ip: 0,
            consts,
        }
    }

    pub fn eval(&mut self) -> Result<(), Error> {
        loop {
            let op = &self.code[self.ip];
            println!("{}|\t{:?}", self.ip, op);
            self.ip += 1;

            match op {
                Opcode::Defslot(s, f) => {
                    self.env.defslot(*s, *f)?;
                }

                Opcode::Assign(s) => {
                    let val = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::eval(concat!(file!(), ":", line!())))?;
                    self.env.assign(s, val)?
                }
                Opcode::Read(s) => {
                    let v = self.env.read(s)?;
                    self.stack.push(v);
                }
                Opcode::LoadField(name) => {
                    let obj = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::eval(concat!(file!(), ":", line!())))?;
                    let field = match obj {
                        Value::Int(_) => todo!(),
                        Value::Uint(_) => todo!(),
                        Value::Float(_) => todo!(),
                        Value::String(_) => todo!(),
                        Value::Str(_) => todo!(),
                        Value::Func(_) => todo!(),
                        Value::Object(obj) => obj
                            .get(name)
                            .cloned()
                            .ok_or_else(|| Error::eval(concat!(file!(), ":", line!())))?,
                        Value::Undefined => todo!(),
                    };
                    self.stack.push(field);
                }
                Opcode::StoreField(_) => todo!(),
                Opcode::Call => match self.stack.pop() {
                    Some(Value::Func(RuntimeFunc::Virtual(func))) => {
                        self.code = Rc::clone(&func.code);
                    }
                    Some(Value::Func(RuntimeFunc::Native(func))) => {
                        func(self);
                    }
                    Some(_) => return Err(Error::eval(concat!(file!(), ":", line!()))),
                    None => return Err(Error::eval(concat!(file!(), ":", line!()))),
                },
                &Opcode::Const(c) => self.stack.push(self.consts[c].into()),

                Opcode::Add => {
                    let b = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::eval(concat!(file!(), ":", line!())))?;
                    let a = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::eval(concat!(file!(), ":", line!())))?;
                    let r = match (a, b) {
                        (Value::Uint(a), Value::Uint(b)) => Value::Uint(a + b),
                        _ => todo!(),
                    };
                    self.stack.push(r);
                }
                Opcode::Sub => todo!(),
                Opcode::Mul => todo!(),
                Opcode::Div => todo!(),
            }

            if self.ip == self.code.len() {
                break;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
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

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Uint(u64),
    Float(f64),
    String(String),
    Str(Spur),
    Func(RuntimeFunc),
    Object(HashMap<Spur, Value>),
    Undefined,
}

#[derive(Clone)]
pub enum RuntimeFunc {
    Native(fn(&mut Vm)),
    Virtual(FuncProto),
}

impl std::fmt::Debug for RuntimeFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native(_) => f.debug_tuple("Native").finish(),
            Self::Virtual(arg0) => f.debug_tuple("Virtual").field(arg0).finish(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ConstValue {
    Int(i64),
    Uint(u64),
    Float(f64),
    Str(Spur),
}

impl From<ConstValue> for Value {
    fn from(cv: ConstValue) -> Value {
        match cv {
            ConstValue::Int(i) => Value::Int(i),
            ConstValue::Uint(u) => Value::Uint(u),
            ConstValue::Float(f) => Value::Float(f),
            ConstValue::Str(s) => Value::Str(s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Opcode {
    Defslot(Spur, Flags),
    Assign(Spur),
    Call,
    Read(Spur),
    LoadField(Spur),
    StoreField(Spur),
    Const(usize),

    Add,
    Sub,
    Mul,
    Div,
}
