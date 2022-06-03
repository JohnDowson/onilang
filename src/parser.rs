use crate::{lexer::Token, BoxedSpannedAst, Span, Spanned, SpannedAst, SpannedAsts};
use chumsky::{
    pratt::{binary_infix_operator, Associativity, InfixOperator, InfixPrecedence},
    prelude::*,
    Parser, Stream,
};

#[derive(Clone)]
pub enum Ast<'s, 'p> {
    Module(SpannedAsts<'s, 'p>),

    Defn(Box<Defn<'s, 'p>>),
    Lambda(BoxedSpannedAst<'s, 'p>, SpannedAsts<'s, 'p>),
    Destruc(Box<Destruc<'s, 'p>>),
    Property(
        BoxedSpannedAst<'s, 'p>,
        Spanned<'p, Token<'s>>,
        BoxedSpannedAst<'s, 'p>,
    ),

    Assignment(Box<Assignment<'s, 'p>>),
    UnaryOp(Spanned<'p, Token<'s>>, Box<SpannedAst<'s, 'p>>),
    BinOp(Box<BinOp<'s, 'p>>),

    String(&'s str),
    Uint(u64),
    Float(f64),

    Loop(Loop<'s, 'p>),

    Call(BoxedSpannedAst<'s, 'p>, BoxedSpannedAst<'s, 'p>),
    New(
        Spanned<'p, Token<'s>>,
        Option<Box<SpannedAst<'s, 'p>>>,
        BoxedSpannedAst<'s, 'p>,
    ),

    Arglist(SpannedAsts<'s, 'p>),
    Paramlist(
        Spanned<'p, Token<'s>>,
        SpannedAsts<'s, 'p>,
        Spanned<'p, Token<'s>>,
    ),

    Identifier(&'s str),

    Access(Box<Access<'s, 'p>>),
}

impl<'s, 'p> std::fmt::Debug for Ast<'s, 'p> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Module(arg0) => f.debug_tuple("Module").field(arg0).finish(),
            Self::Defn(arg0) => arg0.fmt(f),
            Self::Lambda(arg0, arg1) => f.debug_tuple("Lambda").field(arg0).field(arg1).finish(),
            Self::Destruc(arg0) => arg0.fmt(f),
            Self::Property(arg0, arg1, arg2) => f
                .debug_tuple("Property")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::Assignment(arg0) => arg0.fmt(f),
            Self::UnaryOp(arg0, arg1) => f.debug_tuple("UnaryOp").field(arg0).field(arg1).finish(),
            Self::BinOp(arg0) => arg0.fmt(f),
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::Uint(arg0) => f.debug_tuple("Uint").field(arg0).finish(),
            Self::Float(arg0) => f.debug_tuple("Float").field(arg0).finish(),
            Self::Loop(arg0) => arg0.fmt(f),
            Self::Call(arg0, arg1) => f.debug_tuple("Call").field(arg0).field(arg1).finish(),
            Self::New(arg0, arg1, arg2) => f
                .debug_tuple("New")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::Arglist(arg0) => f.debug_tuple("Arglist").field(arg0).finish(),
            Self::Paramlist(arg0, arg1, arg2) => f
                .debug_tuple("Paramlist")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::Identifier(arg0) => f.debug_tuple("Identifier").field(arg0).finish(),
            Self::Access(arg0) => arg0.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Access<'s, 'p> {
    pub base: SpannedAst<'s, 'p>,
    pub accessor: Spanned<'p, Token<'s>>,
    pub field: SpannedAst<'s, 'p>,
}

#[derive(Debug, Clone)]
pub struct Assignment<'s, 'p> {
    pub place: SpannedAst<'s, 'p>,
    pub assign: Spanned<'p, Token<'s>>,
    pub expr: SpannedAst<'s, 'p>,
}

#[derive(Debug, Clone)]
pub struct BinOp<'s, 'p> {
    pub lhs: SpannedAst<'s, 'p>,
    pub op: Spanned<'p, Operator>,
    pub rhs: SpannedAst<'s, 'p>,
}

#[derive(Debug, Clone)]
pub struct Loop<'s, 'p> {
    pub loop_: Spanned<'p, Token<'s>>,
    pub body: SpannedAsts<'s, 'p>,
    pub end: Spanned<'p, Token<'s>>,
}

#[derive(Debug, Clone)]
pub struct Defn<'s, 'p> {
    pub defn: Spanned<'p, Token<'s>>,
    pub name: SpannedAst<'s, 'p>,
    pub args: SpannedAst<'s, 'p>,
    pub _do: Spanned<'p, Token<'s>>,
    pub body: SpannedAsts<'s, 'p>,
    pub end: Spanned<'p, Token<'s>>,
}

#[derive(Debug, Clone)]
pub struct Destruc<'s, 'p> {
    pub destruc: Spanned<'p, Token<'s>>,
    pub name: SpannedAst<'s, 'p>,
    pub _do: Spanned<'p, Token<'s>>,
    pub body: SpannedAsts<'s, 'p>,
    pub end: Spanned<'p, Token<'s>>,
}

fn ident<'s, 'p>(
) -> impl Parser<Token<'s>, SpannedAst<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>> + Clone {
    select! {
        Token::Identifier(i), span =>  Spanned{ span, inner: Ast::Identifier(i) }
    }
}

fn kw_do<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Token<'s>>, Error = Simple<Token<'s>, Span<'p>>> + Clone {
    select! {
        token @ Token::KwDo, span =>  Spanned { span, inner: token }
    }
}

fn pointy<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Token<'s>>, Error = Simple<Token<'s>, Span<'p>>> + Clone {
    select! {
        token @ Token::Pointy, span =>  Spanned { span, inner: token }
    }
}

fn colon<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Token<'s>>, Error = Simple<Token<'s>, Span<'p>>> + Clone {
    select! {
        token @ Token::Colon, span =>  Spanned { span, inner: token }
    }
}

fn kw_defn<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Token<'s>>, Error = Simple<Token<'s>, Span<'p>>> + Clone {
    select! {
        token @ Token::KwDefn, span =>  Spanned { span, inner: token }
    }
}

fn kw_destruc<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Token<'s>>, Error = Simple<Token<'s>, Span<'p>>> + Clone {
    select! {
        token @ Token::KwDestruc, span =>  Spanned { span, inner: token }
    }
}

fn kw_new<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Token<'s>>, Error = Simple<Token<'s>, Span<'p>>> + Clone {
    select! {
        token @ Token::KwNew, span =>  Spanned { span, inner: token }
    }
}

fn kw_end<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Token<'s>>, Error = Simple<Token<'s>, Span<'p>>> + Clone {
    select! {
        token @ Token::KwEnd, span =>  Spanned { span, inner: token }
    }
}

fn kw_loop<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Token<'s>>, Error = Simple<Token<'s>, Span<'p>>> + Clone {
    select! {
        token @ Token::KwLoop, span =>  Spanned { span, inner: token }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
}

impl<'s, 'p> InfixOperator<SpannedAst<'s, 'p>> for Spanned<'p, Operator> {
    type Strength = u8;

    fn precedence(&self) -> InfixPrecedence<Self::Strength> {
        match self.inner {
            Operator::Add => InfixPrecedence::new(0, Associativity::Left),
            Operator::Sub => InfixPrecedence::new(0, Associativity::Left),
            Operator::Mul => InfixPrecedence::new(1, Associativity::Left),
            Operator::Div => InfixPrecedence::new(1, Associativity::Left),
            Operator::Mod => InfixPrecedence::new(1, Associativity::Left),
            Operator::Eq => InfixPrecedence::new(0, Associativity::Left),
            Operator::Ne => InfixPrecedence::new(0, Associativity::Left),
        }
    }

    fn build_expression(
        self,
        lhs: SpannedAst<'s, 'p>,
        rhs: SpannedAst<'s, 'p>,
    ) -> SpannedAst<'s, 'p> {
        Spanned {
            span: Span::merge(&lhs.span, &rhs.span),
            inner: Ast::BinOp(box BinOp { lhs, op: self, rhs }),
        }
    }
}

fn operators<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Operator>, Error = Simple<Token<'s>, Span<'p>>> + Clone {
    select! {
        Token::Plus, span => Spanned { span, inner: Operator::Add},
        Token::Minus, span => Spanned { span, inner: Operator::Sub},
        Token::Star, span => Spanned { span, inner: Operator::Mul},
        Token::Slash, span => Spanned { span, inner: Operator::Div},
        Token::Percent, span => Spanned { span, inner: Operator::Mod},
        Token::Equals, span => Spanned { span, inner: Operator::Eq},
        Token::NotEquals, span => Spanned { span, inner: Operator::Ne},
    }
}

fn expression<'s: 'r, 'p: 'r, 'r>(
) -> impl Parser<Token<'s>, SpannedAst<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>> + 'r + Clone {
    recursive(|expression| {
        let paren_expr = expression
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen))
            .debug("paren_expr");

        let string = select! {
            Token::String(s), span => Spanned { span, inner: Ast::String(s) }
        }
        .debug("string");

        let uint = select! {
            Token::Number(n), span => Spanned { span, inner: Ast::Uint(n.parse().map_err(|e| Simple::custom(span, e))?) }
        }.debug("uint");

        let float = select! {
            Token::Float(n), span => Spanned { span, inner: Ast::Float(n.parse().map_err(|e| Simple::custom(span, e))?) }
        }.debug("float");

        let number = choice((uint, float)).debug("number");

        let literal = choice((string, number)).debug("literal");

        let atom = choice((ident(), literal, paren_expr)).debug("atom");

        let unary = select! { token @ Token::Minus, span => Spanned { span, inner: token } }
            .then(atom.clone())
            .map_with_span(|(op, atom), span| Spanned {
                span,
                inner: Ast::UnaryOp(op, box atom),
            })
            .debug("unary");

        let lambda = arglist()
            .then_ignore(pointy())
            .then(
                expression
                    .clone()
                    .repeated()
                    .delimited_by(just(Token::LCurly), just(Token::RCurly)),
            )
            .map_with_span(|(args, body), span| Spanned {
                span,
                inner: Ast::Lambda(box args, body),
            });

        let accessor = select! {
            token @ Token::Accessor, span => Spanned { span, inner: token }
        }
        .debug("accessor");

        let access = choice((unary.clone(), atom.clone()))
            .then(accessor.then(ident()).repeated().at_least(1))
            .foldl(|base, (accessor, field)| Spanned {
                span: Span::merge(&base.span, &field.span),
                inner: Ast::Access(box Access {
                    base,
                    accessor,
                    field,
                }),
            })
            .debug("access");

        let bin_op = binary_infix_operator(
            choice((access.clone(), unary.clone(), atom.clone())),
            operators(),
        )
        .debug("binop");

        let paramlist = select! { token @ Token::LParen, span => Spanned { span, inner: token} }
            .then(expression.clone().separated_by(just(Token::Comma)))
            .then(select! { token @ Token::RParen, span => Spanned { span, inner: token} })
            .map_with_span(|((lparen, params), rparen), span| Spanned {
                span,
                inner: Ast::Paramlist(lparen, params, rparen),
            });

        let call = choice((access.clone(), ident()))
            .then(paramlist.clone())
            .map_with_span(|(callee, params), span| Spanned {
                span,
                inner: Ast::Call(box callee, box params),
            });

        let new = kw_new()
            .then(paramlist.or_not())
            .then(ident())
            .map_with_span(|((new, params), ty), span| Spanned {
                span,
                inner: Ast::New(new, params.map(|p| box p), box ty),
            });

        let assignment = choice((access.clone(), ident()))
            .then(select! {
                token @ Token::Assign, span => Spanned { span, inner: token },
                token @ Token::ImmutDeclAssign, span => Spanned { span, inner: token },
                token @ Token::DeclAssign, span => Spanned { span, inner: token },
            })
            .then(expression)
            .map_with_span(|((place, assign), expr), span| Spanned {
                span,
                inner: Ast::Assignment(box Assignment {
                    place,
                    assign,
                    expr,
                }),
            })
            .debug("assignment");

        choice((lambda, new, call, access, assignment, bin_op, atom))
    })
}

fn arglist<'s, 'p>(
) -> impl Parser<Token<'s>, SpannedAst<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>> + Clone {
    just(Token::LParen)
        .then(ident().separated_by(just(Token::Comma)))
        .then(just(Token::RParen))
        .map_with_span(|((_lparen, args), _rparen), span| Spanned {
            span,
            inner: Ast::Arglist(args),
        })
}

fn body<'s: 'r, 'p: 'r, 'r>(
) -> impl Parser<Token<'s>, SpannedAsts<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>> + 'r + Clone {
    choice((expression(),)).repeated()
}

fn defn<'s: 'r, 'p: 'r, 'r>(
) -> impl Parser<Token<'s>, SpannedAst<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>> + 'r + Clone {
    kw_defn()
        .then(ident())
        .then(arglist())
        .then(kw_do())
        .then(body())
        .then(kw_end())
        .map_with_span(|(((((defn, name), args), _do), body), end), span| Spanned {
            span,
            inner: Ast::Defn(box Defn {
                defn,
                name,
                args,
                _do,
                body,
                end,
            }),
        })
}

fn destruc<'s: 'r, 'p: 'r, 'r>(
) -> impl Parser<Token<'s>, SpannedAst<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>> + 'r + Clone {
    kw_destruc()
        .then(ident())
        .then(kw_do())
        .then(
            ident()
                .then(colon())
                .then(ident())
                .map_with_span(|((name, colon), ty), span| Spanned {
                    span,
                    inner: Ast::Property(box name, colon, box ty),
                })
                .repeated(),
        )
        .then(kw_end())
        .map_with_span(|((((destruc, name), _do), body), end), span| Spanned {
            span,
            inner: Ast::Destruc(box Destruc {
                destruc,
                name,
                _do,
                body,
                end,
            }),
        })
}

fn implicit_module<'s: 'r, 'p: 'r, 'r>(
) -> impl Parser<Token<'s>, SpannedAst<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>> + 'r {
    choice((defn(), destruc()))
        .repeated()
        .then_ignore(end())
        .map_with_span(|items, span| Spanned {
            span,
            inner: Ast::Module(items),
        })
}

pub fn parse<'s, 'p>(
    tokens: Vec<(Token<'s>, Span<'p>)>,
) -> Result<SpannedAst<'s, 'p>, Vec<Simple<Token<'s>, Span<'p>>>> {
    let stream = Stream::from_iter(tokens.last().unwrap().1, tokens.into_iter());

    implicit_module().parse(stream)
}
