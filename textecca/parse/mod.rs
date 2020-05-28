//! Parsing textecca source.
use nom::error::VerboseError;
use nom_locate::LocatedSpan;

mod cmd;
pub use cmd::*;

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
    indent: Span<'i>,
    text: Span<'i>,
    newline: Span<'i>,
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

/// A parsed but unevaluated region of input.
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'i> {
    /// A region of text, to be output directly.
    Text(Span<'i>),

    /// A command, to be evaluated, and its arguments.
    Command(Command<'i>),
}

/// A function transforming a stream of `RawToken`s into a sequence of `Token`s;
/// that is, parsers decide what delimits a command and how to parse command
/// arguments. In the future, parsers will also decide how to parse sub-blocks.
///
/// This makes textecca's markup language highly flexible, so care must be taken
/// to make parsers that aren't confusing and don't behave unexpectedly.
pub type Parser = for<'i> fn(Vec<RawToken<'i>>) -> Vec<Token<'i>>;
