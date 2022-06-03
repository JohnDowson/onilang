#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(inline_const)]

use parser::Ast;
use std::path::Path;

pub mod compiler;
pub mod error;
pub mod eval;
pub mod lexer;
pub mod parser;

type SpannedAst<'s, 'p> = Spanned<'p, Ast<'s, 'p>>;
type SpannedAsts<'s, 'p> = Vec<Spanned<'p, Ast<'s, 'p>>>;
type BoxedSpannedAst<'s, 'p> = Box<Spanned<'p, Ast<'s, 'p>>>;

#[derive(Clone, Copy)]
pub struct Spanned<'p, T> {
    pub span: Span<'p>,
    pub inner: T,
}

impl<'p, T: std::fmt::Debug> std::fmt::Debug for Spanned<'p, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}@{:#?}", &self.inner, &self.span)
        } else {
            write!(f, "{:?}@{:?}", &self.inner, &self.span)
        }
    }
}

#[derive(Clone, Copy)]
pub struct Span<'p> {
    path: &'p Path,
    start: usize,
    end: usize,
}

impl<'p> Span<'p> {
    pub fn merge(a: &Self, b: &Self) -> Self {
        Self {
            path: a.path,
            start: a.start,
            end: b.end,
        }
    }
}

impl<'p> std::fmt::Debug for Span<'p> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{}..{}@{:?}", self.start, self.end, self.path)
        } else {
            write!(f, "{}..{}", self.start, self.end)
        }
    }
}

impl<'p> chumsky::Span for Span<'p> {
    type Context = &'p Path;

    type Offset = usize;

    fn new(context: Self::Context, range: std::ops::Range<Self::Offset>) -> Self {
        Span {
            path: context,
            start: range.start,
            end: range.end,
        }
    }

    fn context(&self) -> Self::Context {
        self.path
    }

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.end
    }
}
