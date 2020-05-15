#[allow(unused_imports)]
use nom::{
    branch::alt,
    bytes::complete::{tag, take as take_bytes},
    character::complete::{char as take_char, none_of, one_of},
    combinator::{all_consuming, complete, cut, map, not, opt, recognize, rest_len, verify},
    error::{context, ParseError, VerboseError},
    multi::{many0, many1, many1_count, separated_nonempty_list},
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};

use crate::parse::{Error as LexError, Span};

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

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'i> {
    /// A new level of indentation. The span gives the new additional indentation
    /// prefix, which is added to the previous indentation.
    Indent(Span<'i>),

    /// A decrement of some number of indented blocks.
    Deindent(u32),

    /// A word. This is nebulously defined and will be refined over time.
    ///
    /// By default, word boundaries are computed according to [UAX 29][tr29-wb].
    /// However, some grammars may wish to split words into smaller tokens (for
    /// example, while `don't` would be one word in running text, but `f'g` may
    /// be several tokens in an equation context).
    ///
    /// [tr29-wb]: https://unicode.org/reports/tr29/#Default_Word_Boundaries
    Word(Span<'i>),

    /// Inline space, e.g. between words or at the end of a line.
    Space(Span<'i>),

    /// A group of punctuation or symbol codepoints separated by word/space boundaries.
    ///
    /// Punct contains codepoints of the [categories `P` and
    /// `S`][tr44-categories] (punctuation and symbols).
    ///
    /// [tr44-categories]: https://unicode.org/reports/tr44/#General_Category_Values
    Punct(Span<'i>),

    /// A group of number codepoints ([category `N`][tr44-categories]).
    ///
    /// Note that in many cases, a "number" may contain one or more `Num` tokens
    /// surrounded by `Punct` or `Word` tokens (possible edge cases include
    /// strings like `1 million`, `0x33`, `1,000`, `3.22`).
    ///
    /// [tr44-categories]: https://unicode.org/reports/tr44/#General_Category_Values
    Num(Span<'i>),

    /// A line break.
    Newline(Span<'i>),

    /// One or more blank lines.
    BlankLines(BlankLines<'i>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tokens<'i> {
    pub toks: Vec<Token<'i>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Tokenizer<'i> {
    indent: Vec<&'i str>,
}

impl<'i> Tokenizer<'i> {
    fn new() -> Self {
        Default::default()
    }

    /// Recognizes a newline, optionally preceeded by inline whitespace.
    fn newline<E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
        // TODO: Only accept one style of line-ending per-file?
        context(
            "newline",
            recognize(pair(opt(inline_whitespace), alt((tag("\n"), tag("\r\n"))))),
        )(i)
    }

    fn tokenize<E: ParseError<Span<'i>>>(&mut self, input: Span<'i>) -> IResult<Span, (), E> {
        dbg!(input);
        unimplemented!()
    }

    // fn eof<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, (), E> {
    //     not(take_bytes(1usize))(i)
    // }
}
