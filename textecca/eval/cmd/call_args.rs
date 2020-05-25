use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::io::{self, Write};

use thiserror::Error;

use super::param_spec::ParamSpec;
use crate::output::doc::{Block, Blocks};

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

#[derive(Clone, Debug, PartialEq, Error)]
pub enum CommandError {
    #[error("Type error: {0}")]
    TypeError(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedRawArgs {
    pub args: Vec<String>,
    pub kwargs: HashMap<String, String>,
}

pub type FromArgsFn = fn(ParsedRawArgs) -> Result<Box<dyn Command>, FromArgsError>;

pub trait CommandInfo {
    /// The command's name.
    fn name(&self) -> String;

    fn from_args_fn(&self) -> FromArgsFn;
}

pub trait Command {
    fn call(&mut self) -> Result<Blocks, CommandError>;
}
