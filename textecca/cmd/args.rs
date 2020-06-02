use std::collections::{HashMap, VecDeque};
use std::{borrow::Borrow, error};

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

    /// Removes and returns a mandatory argument, either from kwargs, or, if not
    /// given as a keyword argument, from the last positional argument.
    pub fn pop_mandatory(&mut self, name: impl AsRef<str>) -> Result<Thunk<'i>, FromArgsError> {
        self.kwargs
            .remove(name.as_ref())
            .or_else(|| self.args.pop_back())
            .ok_or_else(|| FromArgsError::Missing(name.as_ref().into()))
    }

    /// Returns Err if there are positional or keyword arguments remaining.
    #[must_use]
    pub fn check_no_args(&self) -> Result<(), FromArgsError> {
        self.check_no_posargs()
            .and_then(|()| self.check_no_kwargs())
    }

    /// Returns Err if there are positional arguments remaining.
    #[must_use]
    pub fn check_no_posargs(&self) -> Result<(), FromArgsError> {
        if self.args.is_empty() {
            Ok(())
        } else {
            Err(FromArgsError::TooMany)
        }
    }

    /// Returns Err if there are keyword arguments remaining.
    #[must_use]
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

    /// Missing mandatory argument.
    #[error("Missing a value for argument {0}")]
    Missing(String),

    /// Missing positional argument.
    #[error("Missing a value for positional argument {0}")]
    MissingPositional(String),

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
