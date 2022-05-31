use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Parser error")]
    Parser,
    #[error("Compiler error")]
    Compiler,
    #[error("Eval error")]
    Eval,
}
