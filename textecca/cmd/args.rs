use std::collections::{HashMap, VecDeque};
use std::error;

use thiserror::Error;

use super::{Command, Thunk, World};
use crate::parse::{Argument, Parser};

/// Arguments to a command.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedArgs<'i> {
    /// Positional arguments.
    pub args: VecDeque<Thunk<'i>>,
    /// Keyword arguments.
    pub kwargs: HashMap<String, Thunk<'i>>,
}

impl<'i> ParsedArgs<'i> {
    /// Parse a number of raw arguments (i.e. `Span`s) into a `ParsedArgs`
    /// instance with the given `Parser`, evaluating in the given `World`.
    pub fn from_unparsed(
        args: &[Argument<'i>],
        parser: Parser,
        world: &World<'i>,
    ) -> Result<Self, Box<dyn error::Error + 'i>> {
        let mut posargs = VecDeque::new();
        let mut kwargs = HashMap::new();
        for arg in args {
            // TODO: Handle various errors relating to kwargs in incorrect places.
            let value = parser(world.arena, arg.value)?.into();
            match arg.name {
                Some(kw) => {
                    kwargs.insert(kw.fragment().to_string(), value);
                }
                None => {
                    posargs.push_back(value);
                }
            }
        }
        Ok(ParsedArgs {
            args: posargs,
            kwargs,
        })
    }

    /// Pop the next (i.e. first) positional argument.
    pub fn pop_positional(&mut self) -> Result<Thunk<'i>, FromArgsError> {
        self.args.pop_front().ok_or(FromArgsError::TooFew)
    }

    /// Returns Err if there are positional or keyword arguments remaining.
    pub fn check_no_args(&self) -> Result<(), FromArgsError> {
        self.check_no_posargs()
            .and_then(|()| self.check_no_kwargs())
    }

    /// Returns Err if there are positional arguments remaining.
    pub fn check_no_posargs(&self) -> Result<(), FromArgsError> {
        if self.args.is_empty() {
            Ok(())
        } else {
            Err(FromArgsError::TooMany)
        }
    }

    /// Returns Err if there are keyword arguments remaining.
    pub fn check_no_kwargs(&self) -> Result<(), FromArgsError> {
        if self.kwargs.is_empty() {
            Ok(())
        } else {
            Err(FromArgsError::from_extra_kwargs(self))
        }
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
    /// Too few arguments were given.
    #[error("Too few args")]
    TooFew,

    /// Too many arguments were given.
    #[error("Too many args")]
    TooMany,

    /// A keyword-only argument, mandatory or optional, is used positionally.
    #[error("Arg {0} requires a keyword")]
    MissingKeyword(String),

    /// An unexpected keyword argument was given.
    #[error("Unknown kwarg(s) {0}")]
    UnexpectedKeyword(String),
}

impl FromArgsError {
    /// Create an `UnexpectedKeyword` error from the remaining kwargs in `ParsedArgs`.
    pub fn from_extra_kwargs(parsed: &ParsedArgs<'_>) -> Self {
        FromArgsError::UnexpectedKeyword(itertools::join(
            parsed.kwargs.keys().map(|k| format!("{:?}", k)),
            ",",
        ))
    }
}
