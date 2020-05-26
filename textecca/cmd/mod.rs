//! Commands render particular parts of a document.
//!
//! Commands provide a parser function, which determines how commands and blocks
//! in the command's input are detected.
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::io::{self, Write};

use thiserror::Error;

use crate::doc::{Block, Blocks, DocBuilder};
use crate::env::Environment;
use crate::parse::{Parser, RawTokens, Tokens};

/// Information about a particular command; its name, its parser, and how to construct it.
pub trait CommandInfo {
    /// The command's name.
    fn name(&self) -> String;

    /// A function to create a new instance of the `Command` from arguments.
    fn from_args_fn(&self) -> FromArgs;

    /// The command's argument parser. While the parser for the surrounding
    /// command determines which regions of input represent the arguments to this
    /// command, this parser function is used to determine which regions of input
    /// *within* the arguments refer to other commands and their arguments.
    fn parser_fn(&self) -> Parser;
}

/// A command, which can be called to render itself as blocks to a particular
/// `Serializer`.
pub trait Command {
    fn call(&mut self, env: &Environment, doc: &mut DocBuilder) -> Result<(), CommandError>;
}

/// Arguments to a command.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedArgs<'i> {
    pub args: Vec<Tokens<'i>>,
    pub kwargs: HashMap<String, Tokens<'i>>,
}

/// A `Command` constructor function.
pub type FromArgs = fn(ParsedArgs) -> Result<Box<dyn Command>, FromArgsError>;

/// An error when constructing a `Command` from a `ParsedArgs` instance.
///
/// Errors typically relate to arity mismatches (too few / too many arguments),
/// missing keywords, unknown keyword arguments, etc.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum FromArgsError {
    /// A positional argument after varargs is given.
    #[error("Too few args; given {given} but needed at least {needed}. Missing values for arguments {missing:?}")]
    NotEnoughArgs {
        given: u32,
        needed: u32,
        missing: Vec<String>,
    },

    /// A keyword-only argument, mandatory or optional, is used positionally.
    #[error("Arg {0} requires a keyword")]
    MissingKeyword(String),
}

/// An error while calling a `Command`.
#[derive(Clone, Debug, PartialEq, Error)]
pub enum CommandError {
    #[error("Type error: {0}")]
    TypeError(String),
}
