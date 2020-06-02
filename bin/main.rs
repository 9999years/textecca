#![allow(unused_imports)]
use std::error;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::{convert::TryInto, rc::Rc};

use structopt::StructOpt;
use thiserror::Error;

use textecca::{
    cmd::{CommandError, DefaultCommand, Thunk, World},
    doc::{Block, Doc, DocBuilder, DocBuilderError, DocBuilderPush, Inline},
    env::Environment,
    parse::{default_parser, Source, Span, Token},
    ser::{HtmlSerializer, InitSerializer as _, Serializer as _, SerializerError},
};
use textecca_stdlib as builtins;

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
    Serializer(#[from] SerializerError),

    #[error("{0}")]
    Doc(#[from] DocBuilderError),

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
    let mut env = Environment::new();
    builtins::import(Rc::get_mut(&mut env).unwrap());
    let world = World { env, arena: src };
    let toks = default_parser(src, src.into())?;
    let mut doc = DocBuilder::new();
    Thunk::from(toks).force(&world, &mut doc)?;
    let mut ser = HtmlSerializer::new(io::stdout())?;
    ser.write_doc(doc.try_into()?)?;
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
