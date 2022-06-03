use logos::Logos;

#[derive(Debug, Logos, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token<'s> {
    #[regex("[a-zA-Z$_][a-zA-Z0-9$_]*", |l| l.slice())]
    Identifier(&'s str),

    #[regex("[1-9][0-9]*|0", |l| l.slice())]
    Number(&'s str),
    #[regex(r"[0-9]+\.[0-9]+", |l| l.slice())]
    Float(&'s str),
    #[regex(r#""[a-zA-Z]+""#, |l| l.slice())]
    String(&'s str),

    #[token("let")]
    KwLet,
    #[token("new")]
    KwNew,
    #[token("do")]
    KwDo,
    #[token("end")]
    KwEnd,
    #[token("defn")]
    KwDefn,
    #[token("loop")]
    KwLoop,
    #[token("break")]
    KwBreak,
    #[token("if")]
    KwIf,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("==")]
    Equals,
    #[token("!=")]
    NotEquals,
    #[token(".")]
    Accessor,
    #[token(",")]
    Comma,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(";")]
    Semicolon,

    #[token("$=")]
    ImmutDeclAssign,
    #[token(":=")]
    DeclAssign,
    #[token("=")]
    Assign,

    #[error]
    Error,

    #[regex(r"//.*\n", logos::skip)]
    Comment,
    #[regex(r"\s+", logos::skip)]
    Whitespace,
}
