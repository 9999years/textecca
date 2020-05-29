//! Parsing textecca source.
use std::borrow::Cow;

use nom::{
    combinator::map,
    error::{ParseError, VerboseError},
    IResult,
};
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

/// A parse error.
pub type Error<'input, Extra = ()> = VerboseError<Span<'input, Extra>>;

/// A sequence of `RawToken`s.
pub type RawTokens<'i> = Vec<RawToken<'i>>;
/// A sequence of `Token`s.
pub type Tokens<'i> = Vec<Token<'i>>;

/// A minimally-parsed region of input.
///
/// Note that a sequence of `RawToken`s can be reassembled into the original
/// input, excluding end-of-line whitespace.
///
/// The lifespan `'i` is tied to the parser's input.
#[derive(Debug, Clone, PartialEq)]
pub enum RawToken<'i> {
    Line(Line<'i>),
    BlankLines(BlankLines<'i>),
}

/// A line in the parser input.
#[derive(Debug, Clone, PartialEq)]
pub struct Line<'i> {
    pub indent: Span<'i>,
    pub text: Span<'i>,
    pub newline: Span<'i>,
}

/// A group of one or more blank lines.
/// The lines may include whitespace.
#[derive(Debug, Clone, PartialEq)]
pub struct BlankLines<'i> {
    /// The span encompassing the blank lines, not including the newline that
    /// starts the span of blank lines.
    ///
    /// For example, in the string `"Foo\n\nBar"`, the span would contain only
    /// the second `"\n"`.
    pub span: Span<'i>,

    /// The count of blank lines in the group.
    pub count: u32,
}

impl<'i> From<Line<'i>> for RawToken<'i> {
    fn from(line: Line<'i>) -> Self {
        Self::Line(line)
    }
}

impl<'i> From<BlankLines<'i>> for RawToken<'i> {
    fn from(blanklines: BlankLines<'i>) -> Self {
        Self::BlankLines(blanklines)
    }
}

/// A parsed but unevaluated region of input.
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'i> {
    /// A region of text, to be output directly.
    Text(Span<'i>),

    /// A command, to be evaluated, and its arguments.
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
pub type Parser = for<'i> fn(arena: &'i Source, raw_tokens: RawTokens<'i>) -> Tokens<'i>;

pub fn span_to_tokens<'i, E: ParseError<Span<'i>> + Clone>(
    arena: &'i Source,
    parser: Parser,
) -> impl Fn(Span<'i>) -> IResult<Span<'i>, Tokens<'i>, E> {
    // TODO: make this work and not evil
    |span| map(|i| lex(arena, i), |tokens| parser(arena, tokens))(span)
}
