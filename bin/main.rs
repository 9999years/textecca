#![allow(unused_imports)]
use std::error;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

use structopt::StructOpt;
use thiserror::Error;

use textecca::{
    cmd::{CommandError, World},
    env::Environment,
    parse::{default_parser, Source, Span, Token},
};

#[derive(StructOpt)]
struct Opt {
    /// Input file.
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Error, Debug)]
enum MainError<'i> {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    Command(CommandError<'i>),

    #[error("{0}")]
    Dyn(Box<dyn error::Error + 'i>),
}

impl<'i> From<CommandError<'i>> for MainError<'i> {
    fn from(err: CommandError<'i>) -> Self {
        Self::Command(err)
    }
}

impl<'i> From<Box<dyn error::Error + 'i>> for MainError<'i> {
    fn from(err: Box<dyn error::Error + 'i>) -> Self {
        Self::Dyn(err)
    }
}

fn main_inner<'i>(src: &'i Source) -> Result<(), MainError<'i>> {
    let env = Environment::new();
    let world = World { env, arena: src };
    let toks = default_parser(src, src.into())?;
    for tok in toks {
        match tok {
            Token::Text(s) => {
                print!("{}", s.fragment());
            }
            Token::Command(cmd) => {
                println!("\n{:?}", world.get_cmd(cmd)?);
            }
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    let mut input = String::new();
    let mut fh = File::open(opt.input)?;
    fh.read_to_string(&mut input)?;
    let src = Source::new(input);
    if let Err(err) = main_inner(&src) {
        println!("\nError: {}", err);
        println!("Debug: {:#?}", err);
    }
    Ok(())
}
