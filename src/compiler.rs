use std::{collections::HashMap, rc::Rc};

use lasso::{Rodeo, Spur};

use crate::{
    error::Error,
    eval::{ConstValue, Flags, Opcode, Vm},
    parser::{Ast, Defn},
    Spanned, SpannedAst,
};

#[derive(Clone, Debug)]
pub struct FuncProto {
    pub code: Rc<[Opcode]>,
}

struct IncompleteFuncProto {
    pub code: Vec<Opcode>,
}

impl IncompleteFuncProto {
    fn finalize(self) -> FuncProto {
        let code = self.code.into();
        FuncProto { code }
    }
}

pub struct Scope {}

pub struct Compiler<'i> {
    funcs: HashMap<Spur, FuncProto>,
    interner: &'i mut Rodeo,
    consts: Vec<ConstValue>,
}

impl<'i> Compiler<'i> {
    pub fn compile<'s, 'p>(
        ast: SpannedAst<'s, 'p>,
        interner: &'i mut Rodeo,
    ) -> Result<Vm<'i>, Error> {
        let mut this = Self {
            funcs: Default::default(),
            consts: Default::default(),
            interner,
        };
        let items = match ast.inner {
            Ast::Module(items) => items,
            _ => return Err(Error::Compiler),
        };
        for item in items {
            this.compile_item(item)?;
        }
        Ok(this.emit())
    }

    fn compile_item<'s, 'p>(&mut self, item: SpannedAst<'s, 'p>) -> Result<(), Error> {
        match item.inner {
            Ast::Module(_) => todo!(),
            Ast::Defn(box defn) => {
                let Defn {
                    defn: _,
                    name,
                    args,
                    _do,
                    body,
                    end: _,
                } = defn;
                let name = if let Ast::Identifier(ident) = name.inner {
                    self.interner.get_or_intern(ident)
                } else {
                    return Err(Error::Compiler);
                };

                let mut func = IncompleteFuncProto {
                    code: Default::default(),
                };

                let args = if let Ast::Arglist(args) = args.inner {
                    args
                } else {
                    return Err(Error::Compiler);
                };
                for arg in args {
                    let arg = if let Ast::Identifier(ident) = arg.inner {
                        self.interner.get_or_intern(ident)
                    } else {
                        return Err(Error::Compiler);
                    };
                    func.code
                        .push(Opcode::Defslot(arg, Flags::BINDING_MODE_IMMUT));
                }

                for expr in body {
                    self.compile_expr(&mut func, expr)?;
                }

                self.funcs.insert(name, func.finalize());
                Ok(())
            }
            _ => Err(Error::Compiler),
        }
    }

    fn emit(self) -> Vm<'i> {
        let code = Rc::clone(
            &self
                .funcs
                .get(&self.interner.get("main").unwrap())
                .unwrap()
                .code,
        );
        Vm::new(code, self.consts, self.interner)
    }

    fn compile_expr(
        &mut self,
        func: &mut IncompleteFuncProto,
        expr: Spanned<Ast>,
    ) -> Result<(), Error> {
        match expr.inner {
            Ast::Module(_) => todo!(),
            Ast::Defn(_) => todo!(),
            Ast::Assignment(_) => todo!(),
            Ast::String(s) => {
                let s = self.interner.get_or_intern(s);
                self.consts.push(ConstValue::Str(s));
                func.code.push(Opcode::Const(self.consts.len() - 1));
            }
            Ast::Int(i) => {
                self.consts.push(ConstValue::Int(i));
                func.code.push(Opcode::Const(self.consts.len() - 1));
            }
            Ast::Uint(i) => {
                self.consts.push(ConstValue::Uint(i));
                func.code.push(Opcode::Const(self.consts.len() - 1));
            }
            Ast::Call(
                box Spanned {
                    span: _name_span,
                    inner: Ast::Identifier(name),
                },
                box Spanned {
                    span: _params_span,
                    inner: Ast::Paramlist(params),
                },
            ) => {
                let name = self.interner.get_or_intern(name);
                for param in params {
                    self.compile_expr(func, param)?;
                }
                func.code.push(Opcode::Call(name));
            }
            Ast::Call(..) => unimplemented!(),
            Ast::New(_, _, _) => todo!(),
            Ast::Arglist(_) => todo!(),
            Ast::Paramlist(_) => todo!(),
            Ast::Identifier(_) => todo!(),
            Ast::Place(_, _) => todo!(),
        }
        Ok(())
    }
}
