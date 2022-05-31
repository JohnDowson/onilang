use std::collections::HashMap;

use crate::{
    error::Error,
    eval::{Opcode, Value},
    parser::{Ast, Defn},
    Spanned, SpannedAst,
};

pub struct Func<'i> {
    code: Vec<Opcode<'i>>,
}

pub struct Scope {}

pub struct Compiler<'i> {
    funcs: HashMap<String, Func<'i>>,
    constants: Vec<Value>,
}

impl<'i> Compiler<'i> {
    pub fn compile<'s, 'p>(
        ast: SpannedAst<'s, 'p>,
    ) -> Result<(Vec<Opcode<'i>>, Vec<Value>), Error> {
        let mut this = Self {
            funcs: Default::default(),
            constants: Default::default(),
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
                    defn,
                    name,
                    args,
                    _do,
                    body,
                    end,
                } = defn;
                let name = if let Ast::Identifier(i) = name.inner {
                    i.to_owned()
                } else {
                    return Err(Error::Compiler);
                };
                let mut func = Func {
                    code: Default::default(),
                };

                for expr in body {
                    self.compile_expr(&mut func, expr)?;
                }

                self.funcs.insert(name, func);
                Ok(())
            }
            _ => Err(Error::Compiler),
        }
    }

    fn emit(self) -> (Vec<Opcode<'i>>, Vec<Value>) {
        todo!()
    }

    fn compile_expr(&mut self, func: &mut Func, expr: Spanned<Ast>) -> Result<(), Error> {
        match expr.inner {
            Ast::Module(_) => todo!(),
            Ast::Defn(_) => todo!(),
            Ast::Assignment(_) => todo!(),
            Ast::String(s) => {
                self.constants.push(Value::String(()));
                func.code.push(Opcode::Const(self.constants.len() - 1));
            }
            Ast::Int(i) => {
                self.constants.push(Value::Int(i));
                func.code.push(Opcode::Const(self.constants.len() - 1));
            }
            Ast::Uint(i) => {
                self.constants.push(Value::Uint(i));
                func.code.push(Opcode::Const(self.constants.len() - 1));
            }
            Ast::Call(
                box Spanned {
                    span: name_span,
                    inner: Ast::Identifier(name),
                },
                box Spanned {
                    span: params_span,
                    inner: Ast::Paramlist(params),
                },
            ) => {
                for param in params {
                    self.compile_expr(func, param)?;
                }
                todo!()
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
