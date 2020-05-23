use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use thiserror::Error;

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

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedRawArgs {
    pub args: Vec<String>,
    pub kwargs: HashMap<String, String>,
}

pub trait Command: TryFrom<ParsedRawArgs, Error = FromArgsError> {
    /// The command's name.
    fn name() -> String;
}
