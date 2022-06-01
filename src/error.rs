use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("Parser error")]
    Parser,
    #[error("Compiler error")]
    Compiler,
    #[error("Eval error")]
    Eval,
}

#[derive(Debug, Error)]
#[error("{}@{}", .kind, .location)]
pub struct Error {
    location: &'static str,
    kind: ErrorKind,
}

impl Error {
    pub fn parser(location: &'static str) -> Self {
        Self {
            location,
            kind: ErrorKind::Parser,
        }
    }

    pub fn compiler(location: &'static str) -> Self {
        Self {
            location,
            kind: ErrorKind::Compiler,
        }
    }

    pub fn eval(location: &'static str) -> Self {
        Self {
            location,
            kind: ErrorKind::Eval,
        }
    }
}
