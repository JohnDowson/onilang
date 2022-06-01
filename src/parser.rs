use crate::{lexer::Token, BoxedSpannedAst, Span, Spanned, SpannedAst, SpannedAsts};
use chumsky::{prelude::*, Parser, Stream};

#[derive(Debug)]
pub enum Ast<'s, 'p> {
    Module(SpannedAsts<'s, 'p>),

    Defn(Box<Defn<'s, 'p>>),

    Assignment(Box<Assignment<'s, 'p>>),
    BinOp(Box<BinOp<'s, 'p>>),

    String(&'s str),
    Int(i64),
    Uint(u64),

    Loop(Loop<'s, 'p>),

    Call(BoxedSpannedAst<'s, 'p>, BoxedSpannedAst<'s, 'p>),
    New(
        Spanned<'p, Token<'s>>,
        Option<Box<SpannedAst<'s, 'p>>>,
        BoxedSpannedAst<'s, 'p>,
    ),

    Arglist(SpannedAsts<'s, 'p>),
    Paramlist(SpannedAsts<'s, 'p>),

    Identifier(&'s str),

    Place(BoxedSpannedAst<'s, 'p>, SpannedAsts<'s, 'p>),
}

#[derive(Debug)]
pub struct Assignment<'s, 'p> {
    pub place: SpannedAst<'s, 'p>,
    pub assign: Spanned<'p, Token<'s>>,
    pub expr: SpannedAst<'s, 'p>,
}

#[derive(Debug)]
pub struct BinOp<'s, 'p> {
    pub lhs: SpannedAst<'s, 'p>,
    pub op: Spanned<'p, Token<'s>>,
    pub rhs: SpannedAst<'s, 'p>,
}

#[derive(Debug)]
pub struct Loop<'s, 'p> {
    pub loop_: Spanned<'p, Token<'s>>,
    pub body: SpannedAsts<'s, 'p>,
    pub end: Spanned<'p, Token<'s>>,
}

#[derive(Debug)]
pub struct Defn<'s, 'p> {
    pub defn: Spanned<'p, Token<'s>>,
    pub name: SpannedAst<'s, 'p>,
    pub args: SpannedAst<'s, 'p>,
    pub _do: Spanned<'p, Token<'s>>,
    pub body: SpannedAsts<'s, 'p>,
    pub end: Spanned<'p, Token<'s>>,
}

fn ident<'s, 'p>() -> impl Parser<Token<'s>, SpannedAst<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>>
{
    select! {
        Token::Identifier(i), span =>  Spanned{ span, inner: Ast::Identifier(i) }
    }
}

fn kw_do<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Token<'s>>, Error = Simple<Token<'s>, Span<'p>>> {
    select! {
        token @ Token::KwDo, span =>  Spanned { span, inner: token }
    }
}

fn kw_defn<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Token<'s>>, Error = Simple<Token<'s>, Span<'p>>> {
    select! {
        token @ Token::KwDefn, span =>  Spanned { span, inner: token }
    }
}

fn kw_new<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Token<'s>>, Error = Simple<Token<'s>, Span<'p>>> {
    select! {
        token @ Token::KwNew, span =>  Spanned { span, inner: token }
    }
}

fn kw_end<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Token<'s>>, Error = Simple<Token<'s>, Span<'p>>> {
    select! {
        token @ Token::KwEnd, span =>  Spanned { span, inner: token }
    }
}

fn kw_loop<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Token<'s>>, Error = Simple<Token<'s>, Span<'p>>> {
    select! {
        token @ Token::KwLoop, span =>  Spanned { span, inner: token }
    }
}

fn equals<'s, 'p>(
) -> impl Parser<Token<'s>, Spanned<'p, Token<'s>>, Error = Simple<Token<'s>, Span<'p>>> {
    select! {
        token @ Token::Equals, span =>  Spanned { span, inner: token }
    }
}

fn place<'s, 'p>() -> impl Parser<Token<'s>, SpannedAst<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>>
{
    ident()
        .then(just(Token::Accessor).ignore_then(ident()).repeated())
        .map_with_span(|(base, accessors), span| Spanned {
            span,
            inner: Ast::Place(box base, accessors),
        })
}

fn expression<'s: 'r, 'p: 'r, 'r>(
) -> impl Parser<Token<'s>, SpannedAst<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>> + 'r + Clone {
    recursive(|expression| {
        let paramlist = just(Token::LParen)
            .then(expression.clone().separated_by(just(Token::Comma)))
            .then(just(Token::RParen))
            .map_with_span(|((_lparen, args), _rparen), span| Spanned {
                span,
                inner: Ast::Paramlist(args),
            });

        let bin_op = expression
            .clone()
            .then(choice((equals(),)))
            .then(expression.clone())
            .map_with_span(|((lhs, op), rhs), span| Spanned {
                span,
                inner: Ast::BinOp(box BinOp { lhs, op, rhs }),
            });

        let loop_ = kw_loop()
            .then(expression.repeated())
            .then(kw_end())
            .map_with_span(|((loop_, body), end), span| Spanned {
                span,
                inner: Ast::Loop(Loop { loop_, body, end }),
            });

        let string = select! {
            Token::String(s), span => Spanned { span, inner: Ast::String(s) }
        };

        let uint = select! {
            Token::Number(n), span => Spanned { span, inner: Ast::Uint(n) }
        };

        let int = just(Token::Minus).ignore_then(select! {
            Token::Number(n), span => Spanned { span, inner: Ast::Int(-(n as i64)) }
        });

        let new = kw_new()
            .then(paramlist.clone().or_not())
            .then(ident())
            .map_with_span(|((new, params), ty), span| Spanned {
                span,
                inner: Ast::New(new, params.map(|p| box p), box ty),
            });

        let call = ident()
            .then(paramlist)
            .map_with_span(|(name, params), span| Spanned {
                span,
                inner: Ast::Call(box name, box params),
            });

        let number = choice((uint, int));

        let literal = choice((string, number));

        choice((literal, new, call, place(), loop_, bin_op))
    })
}

fn assignment<'s: 'r, 'p: 'r, 'r>(
) -> impl Parser<Token<'s>, SpannedAst<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>> + 'r {
    place()
        .then(select! {
            token @ Token::Assign, span => Spanned { span, inner: token },
            token @ Token::ImmutDeclAssign, span => Spanned { span, inner: token },
            token @ Token::DeclAssign, span => Spanned { span, inner: token },
        })
        .then(expression())
        .map_with_span(|((place, assign), expr), span| Spanned {
            span,
            inner: Ast::Assignment(box Assignment {
                place,
                assign,
                expr,
            }),
        })
}

fn arglist<'s, 'p>(
) -> impl Parser<Token<'s>, SpannedAst<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>> {
    just(Token::LParen)
        .then(ident().separated_by(just(Token::Comma)))
        .then(just(Token::RParen))
        .map_with_span(|((_lparen, args), _rparen), span| Spanned {
            span,
            inner: Ast::Arglist(args),
        })
}

fn body<'s: 'r, 'p: 'r, 'r>(
) -> impl Parser<Token<'s>, SpannedAsts<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>> + 'r {
    choice((assignment(), expression())).repeated()
}

fn defn<'s: 'r, 'p: 'r, 'r>(
) -> impl Parser<Token<'s>, SpannedAst<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>> + 'r {
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

fn implicit_module<'s: 'r, 'p: 'r, 'r>(
) -> impl Parser<Token<'s>, SpannedAst<'s, 'p>, Error = Simple<Token<'s>, Span<'p>>> + 'r {
    choice((defn(),))
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
