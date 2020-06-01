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

use crate::doc::{Block, Blocks, DocBuilder, DocBuilderError};
use crate::env::Environment;
use crate::parse::{self, Argument, Parser, Source, Tokens};

mod args;
mod default_cmd;
mod thunk;

pub use args::*;
pub use default_cmd::*;
pub use thunk::*;

/// Memoized information about a particular command; its name, its parser, and
/// how to construct it.
///
/// See also: `CommandInfo`.
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct CommandInfoMemo {
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

impl CommandInfoMemo {
    /// Create a new `CommandInfoMemo` from the given type.
    pub fn new<C: CommandInfo>() -> Self {
        Self {
            name: C::name(),
            from_args_fn: C::from_args_fn(),
            parser_fn: C::parser_fn(),
        }
    }
}

/// Information about a particular command.
pub trait CommandInfo {
    /// The command's name.
    fn name() -> String;
    /// The command's initializer function.
    fn from_args_fn() -> FromArgs;
    /// The command's embedded parser for interpreting arguments.
    ///
    /// Currently defaults to `parse::default_parser` but should be inherited
    /// from the surrounding command.
    fn parser_fn() -> Parser {
        parse::default_parser
    }
}

/// A command, which can be called to render itself as blocks to a particular
/// `Serializer`.
pub trait Command<'i>: std::fmt::Debug {
    /// Call (i.e. evaluate) the given `Command`.
    fn call(
        self: Box<Self>,
        doc: &mut DocBuilder,
        world: &World<'i>,
    ) -> Result<(), CommandError<'i>>;

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
    /// The environment of bindings.
    pub env: Rc<Environment>,
    /// The arena, for generating new tokens.
    pub arena: &'i Source,
}

impl<'i> World<'i> {
    /// Construct the given `Command` and parse its arguments.
    pub fn get_cmd(
        &self,
        cmd: parse::Command<'i>,
    ) -> Result<Box<dyn Command<'i> + 'i>, CommandError<'i>> {
        let name = *cmd.name.fragment();
        let info = self.env.cmd_info(name)?;
        let mut args = ParsedArgs::from_unparsed(&cmd.args, info.parser_fn, self)
            .map_err(CommandError::ParseError)?;
        Ok((info.from_args_fn)(&mut args)?)
    }

    /// Construct and call the given `Command`.
    pub fn call_cmd(
        &self,
        cmd: parse::Command<'i>,
        doc: &mut DocBuilder,
    ) -> Result<(), CommandError<'i>> {
        self.get_cmd(cmd)?.call(doc, self)
    }
}

/// An error while calling a `Command`.
#[derive(Debug, Error)]
pub enum CommandError<'i> {
    /// A type error.
    #[error("Type error: {0}")]
    Type(String),

    /// An error while initializing the `Command` from a `ParsedArgs` instance.
    #[error("Args error: {0}")]
    FromArgs(#[from] FromArgsError),

    /// An unbound command.
    #[error("Command {0} not defined in current environment")]
    Name(String),

    /// An error while parsing the `Command`'s arguments.
    #[error("Parse error: {0}")]
    ParseError(Box<dyn error::Error + 'i>),

    /// Error while creating the output document.
    #[error("{0}")]
    DocBuilder(#[from] DocBuilderError),
}
