use std::{collections::HashMap, rc::Rc};

use lasso::{Rodeo, Spur};

use crate::{
    error::Error,
    eval::{ConstValue, Flags, Opcode, Vm},
    lexer::Token,
    parser::{Assignment, Ast, Defn},
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
            _ => return Err(Error::compiler(concat!(file!(), ":", line!()))),
        };
        for item in items {
            this.compile_item(item)?;
        }
        this.emit()
    }

    fn compile_item(&mut self, item: SpannedAst<'_, '_>) -> Result<(), Error> {
        match item.inner {
            Ast::Module(_) => todo!(),
            Ast::Defn(box defn) => self.compile_defn(defn),
            _ => Err(Error::compiler(concat!(file!(), ":", line!()))),
        }
    }

    fn compile_defn(&mut self, defn: Defn) -> Result<(), Error> {
        let Defn {
            defn: _,
            name,
            args,
            _do: _,
            body,
            end: _,
        } = defn;
        let name = if let Ast::Identifier(ident) = name.inner {
            self.interner.get_or_intern(ident)
        } else {
            return Err(Error::compiler(concat!(file!(), ":", line!())));
        };

        let mut func = IncompleteFuncProto {
            code: Default::default(),
        };

        let args = if let Ast::Arglist(args) = args.inner {
            args
        } else {
            return Err(Error::compiler(concat!(file!(), ":", line!())));
        };
        for arg in args {
            let arg = if let Ast::Identifier(ident) = arg.inner {
                self.interner.get_or_intern(ident)
            } else {
                return Err(Error::compiler(concat!(file!(), ":", line!())));
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

    fn compile_expr(
        &mut self,
        func: &mut IncompleteFuncProto,
        expr: Spanned<Ast>,
    ) -> Result<(), Error> {
        match expr.inner {
            Ast::Module(_) => Err(Error::compiler(concat!(file!(), ":", line!()))),
            Ast::Defn(_) => Err(Error::compiler(concat!(file!(), ":", line!()))),
            Ast::Assignment(box Assignment {
                place:
                    Spanned {
                        span: _,
                        inner:
                            Ast::Place(
                                box Spanned {
                                    span: _,
                                    inner: Ast::Identifier(object_ident),
                                },
                                mut accessors,
                            ),
                    },
                assign,
                expr,
            }) => {
                let object_name = self.interner.get_or_intern(object_ident);
                match assign.inner {
                    Token::ImmutDeclAssign => func
                        .code
                        .push(Opcode::Defslot(object_name, Flags::BINDING_MODE_IMMUT)),
                    Token::DeclAssign => func
                        .code
                        .push(Opcode::Defslot(object_name, Flags::BINDING_MODE_MUT)),
                    Token::Assign => (),
                    _ => return Err(Error::compiler(concat!(file!(), ":", line!()))),
                }
                if let Some(Spanned {
                    span: _,
                    inner: Ast::Identifier(field_ident),
                }) = accessors.pop()
                {
                    func.code.push(Opcode::Read(object_name));
                    let last_field_name = self.interner.get_or_intern(field_ident);

                    for accessor in accessors {
                        if let Spanned {
                            span: _,
                            inner: Ast::Identifier(ident),
                        } = accessor
                        {
                            let name = self.interner.get_or_intern(ident);
                            func.code.push(Opcode::LoadField(name));
                        } else {
                            return Err(Error::compiler(concat!(file!(), ":", line!())));
                        }
                    }

                    self.compile_expr(func, expr)?;

                    func.code.push(Opcode::StoreField(last_field_name));
                } else {
                    self.compile_expr(func, expr)?;
                    func.code.push(Opcode::Assign(object_name));
                };

                Ok(())
            }
            Ast::Assignment(_) => Err(Error::compiler(concat!(file!(), ":", line!()))),
            Ast::BinOp(_) => todo!(),
            Ast::String(s) => {
                let s = self.interner.get_or_intern(s);
                self.consts.push(ConstValue::Str(s));
                func.code.push(Opcode::Const(self.consts.len() - 1));
                Ok(())
            }
            Ast::Int(i) => {
                self.consts.push(ConstValue::Int(i));
                func.code.push(Opcode::Const(self.consts.len() - 1));
                Ok(())
            }
            Ast::Uint(i) => {
                self.consts.push(ConstValue::Uint(i));
                func.code.push(Opcode::Const(self.consts.len() - 1));
                Ok(())
            }
            Ast::Loop(_) => todo!(),
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
                Ok(())
            }
            Ast::Call(..) => unimplemented!(),
            Ast::New(_, _, _) => todo!(),
            Ast::Arglist(_) => todo!(),
            Ast::Paramlist(_) => todo!(),
            Ast::Identifier(_) => todo!(),
            Ast::Place(
                box Spanned {
                    span: _,
                    inner: Ast::Identifier(ident),
                },
                _accessors,
            ) => {
                let ident = self.interner.get_or_intern(ident);
                func.code.push(Opcode::Read(ident));
                Ok(())
            }
            Ast::Place(..) => Err(Error::compiler(concat!(file!(), ":", line!()))),
        }
    }

    fn emit(self) -> Result<Vm<'i>, Error> {
        let code = Rc::clone(
            &self
                .funcs
                .get(&self.interner.get("main").unwrap())
                .ok_or_else(|| Error::compiler(concat!(file!(), ":", line!())))?
                .code,
        );
        Ok(Vm::new(code, self.consts, self.interner))
    }
}
