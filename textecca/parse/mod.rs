//! Parsing textecca source.
use std::borrow::Cow;
use std::error::Error;

use nom_locate::LocatedSpan;

mod arena;
mod cmd;
mod default_parser;
mod lex;
mod parse_util;
mod ucd_tables;

#[macro_use]
#[cfg(test)]
mod test_util;

pub use arena::*;
pub use cmd::*;
pub use default_parser::*;
pub use lex::*;

/// A region of input.
///
/// The lifespan `'i` is tied to the parser's input, e.g. the file's contents in
/// memory.
pub type Span<'input, Extra = ()> = LocatedSpan<&'input str, Extra>;

/// A sequence of `Token`s.
pub type Tokens<'i> = Vec<Token<'i>>;

/// A parsed but unevaluated region of input.
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'i> {
    /// A region of text, to be output directly.
    Text(Span<'i>),

    /// A command to be evaluated along with its arguments.
    Command(Command<'i>),
}

impl<'i> From<Span<'i>> for Token<'i> {
    fn from(span: Span<'i>) -> Self {
        Self::Text(span)
    }
}

impl<'i> From<Command<'i>> for Token<'i> {
    fn from(cmd: Command<'i>) -> Self {
        Self::Command(cmd)
    }
}

/// A function transforming a stream of `RawToken`s into a sequence of `Token`s;
/// that is, parsers decide what delimits a command and how to parse command
/// arguments. In the future, parsers will also decide how to parse sub-blocks.
///
/// This makes textecca's markup language highly flexible, so care must be taken
/// to make parsers that aren't confusing and don't behave unexpectedly.
pub type Parser =
    for<'i> fn(arena: &'i Source, input: Span<'i>) -> Result<Tokens<'i>, Box<dyn Error>>;
