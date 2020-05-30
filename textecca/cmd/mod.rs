//! Commands render particular parts of a document.
//!
//! Commands provide a parser function, which determines how commands and blocks
//! in the command's input are detected.
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::error;
use std::io::{self, Write};
use std::rc::Rc;

use derivative::Derivative;
use thiserror::Error;

use crate::doc::{Block, Blocks, DocBuilder};
use crate::env::Environment;
use crate::parse::{self, Argument, Parser, RawTokens, Source, Tokens};

mod default_cmd;
mod thunk;

pub use default_cmd::*;
pub use thunk::*;

/// Information about a particular command; its name, its parser, and how to construct it.
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct CommandInfo {
    /// The command's name.
    pub name: String,
    /// A function to create a new instance of the `Command` from arguments.
    #[derivative(Debug = "ignore")]
    pub from_args_fn: FromArgs,
    /// The command's argument parser. While the parser for the surrounding
    /// command determines which regions of input represent the arguments to this
    /// command, this parser function is used to determine which regions of input
    /// *within* the arguments refer to other commands and their arguments.
    #[derivative(Debug = "ignore")]
    pub parser_fn: Parser,
}

impl CommandInfo {
    fn new(name: String, from_args_fn: FromArgs, parser_fn: Parser) -> Self {
        Self {
            name,
            from_args_fn,
            parser_fn,
        }
    }

    fn from_name_and_args(name: String, from_args_fn: FromArgs) -> Self {
        Self::new(name, from_args_fn, parse::default_parser)
    }
}

/// A command, which can be called to render itself as blocks to a particular
/// `Serializer`.
pub trait Command<'i> {
    /// Call (i.e. evaluate) the given `Command`.
    fn call(self: Box<Self>, world: &World<'i>) -> Result<Blocks, CommandError<'i>>;

    /// Get the environment this command's arguments are evaluated in.
    ///
    /// For example, if this command's `Parser` transformed a `-` at the
    /// beginning of a line into `\item`, the returned environment should have
    /// `\item` bound.
    fn environment(&self, parent: Rc<Environment>) -> Result<Rc<Environment>, CommandError> {
        Ok(Environment::new_inheriting(parent))
    }
}

/// An evaluation context for `Command`s.
#[derive(Debug, Clone)]
pub struct World<'i> {
    pub env: Rc<Environment>,
    pub arena: &'i Source,
}

/// Arguments to a command.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedArgs<'i> {
    pub args: Vec<Thunk<'i>>,
    pub kwargs: HashMap<String, Thunk<'i>>,
}

impl<'i> ParsedArgs<'i> {
    pub fn from_unparsed(
        args: &[Argument<'i>],
        parser: Parser,
        world: &World<'i>,
    ) -> Result<Self, Box<dyn error::Error + 'i>> {
        let mut posargs = Vec::new();
        let mut kwargs = HashMap::new();
        for arg in args {
            // TODO: Handle various errors relating to kwargs in incorrect places.
            let value = parser(world.arena, arg.value)?.into();
            match arg.name {
                Some(kw) => {
                    kwargs.insert(kw.fragment().to_string(), value);
                }
                None => {
                    posargs.push(value);
                }
            }
        }
        Ok(ParsedArgs {
            args: posargs,
            kwargs,
        })
    }
}

/// A `Command` constructor function.
pub type FromArgs =
    for<'i> fn(&mut ParsedArgs<'i>) -> Result<Box<dyn Command<'i> + 'i>, FromArgsError>;

/// An error when constructing a `Command` from a `ParsedArgs` instance.
///
/// Errors typically relate to arity mismatches (too few / too many arguments),
/// missing keywords, unknown keyword arguments, etc.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum FromArgsError {
    #[error("Too few args")]
    NotEnough,

    #[error("Too many args")]
    TooMany,

    /// A keyword-only argument, mandatory or optional, is used positionally.
    #[error("Arg {0} requires a keyword")]
    MissingKeyword(String),

    #[error("Unknown kwarg(s) {0}")]
    UnexpectedKeyword(String),
}

impl FromArgsError {
    pub fn from_extra_kwargs(parsed: &ParsedArgs<'_>) -> Self {
        FromArgsError::UnexpectedKeyword(itertools::join(
            parsed.kwargs.keys().map(|k| format!("{:?}", k)),
            ",",
        ))
    }
}

/// An error while calling a `Command`.
#[derive(Debug, Error)]
pub enum CommandError<'i> {
    #[error("Type error: {0}")]
    Type(String),

    #[error("Args error: {0}")]
    FromArgs(#[from] FromArgsError),

    #[error("Command {0} not defined in current environment")]
    Name(String),

    #[error("Parse error: {0}")]
    ParseError(Box<dyn error::Error + 'i>),
}
