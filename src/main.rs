use chumsky::Span as _;
use clap::Parser;
use lasso::Rodeo;
use logos::Logos;
use onilang::{compiler::Compiler, error::Error, lexer::Token, parser::parse, Span};
use std::{fs::File, io::Read, path::PathBuf};

#[derive(Parser)]
struct Args {
    #[clap()]
    file: PathBuf,
    #[clap(short = 't', long)]
    dump_tokens: bool,
    #[clap(short = 'a', long)]
    dump_ast: bool,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let mut file = File::open(&args.file).unwrap();
    let mut src = String::new();
    file.read_to_string(&mut src).unwrap();

    let tokens = Token::lexer(&src)
        .spanned()
        .map(|(t, s)| (t, Span::new(&args.file, s)))
        .collect::<Vec<_>>();
    if args.dump_tokens {
        println!("{:#?}", &tokens);
    }

    let ast = match parse(tokens) {
        Ok(ast) => ast,
        Err(errors) => {
            for e in errors {
                eprintln!("{:?}", e);
            }
            return Err(Error::Parser);
        }
    };
    if args.dump_ast {
        println!("{:#?}", &ast);
    }

    let mut interner = Rodeo::new();
    let mut vm = Compiler::compile(ast, &mut interner)?;
    vm.eval()?;

    Ok(())
}
