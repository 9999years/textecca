use nom::{
    branch::*,
    bytes::complete::{tag, take as take_bytes},
    bytes::streaming::{take_while, take_while1},
    character::complete::{anychar, char as take_char, line_ending, none_of, one_of},
    combinator::*,
    error::{context, make_error, ErrorKind, ParseError, VerboseError},
    multi::*,
    sequence::*,
    ExtendInto, IResult, Slice,
};

use super::parse_util::*;
use super::{Source, Span};

/// A sequence of `RawToken`s.
pub type RawTokens<'i> = Vec<RawToken<'i>>;

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

/// Transform `&str` input into `RawTokens`.
pub fn lex<'i, E: ParseError<Span<'i>> + Clone>(
    src: &'i Source,
    input: Span<'i>,
) -> IResult<Span<'i>, RawTokens<'i>, E> {
    let mut it = iterator(
        input,
        map(
            tuple((
                recognize(many0(one_of(" \t"))),    // indent
                recognize(many0(none_of("\r\n"))),  // line content
                alt((recognize(eof), line_ending)), // newline
            )),
            |(indent, text, newline)| line_into_rawtoken(&input, indent, text, newline),
        ),
    );
    let mut ret = Vec::with_capacity(input.fragment().len() / 80);
    for raw_token in &mut it {
        ret.push(raw_token);
        merge_last_blanklines(src, &mut ret);
    }
    it.finish().map(|(rest, ())| (rest, ret))
}

fn line_into_rawtoken<'i>(
    input: &Span<'i>,
    indent: Span<'i>,
    text: Span<'i>,
    newline: Span<'i>,
) -> RawToken<'i> {
    if text.fragment().chars().all(is_inline_space) {
        // Rationale: indent, text, and newline are adjacent in the source input.
        let span = input.slice(
            indent.location_offset()
                ..indent.fragment().len() + text.fragment().len() + newline.fragment().len(),
        );
        BlankLines { span, count: 1 }.into()
    } else {
        Line {
            indent,
            text,
            newline,
        }
        .into()
    }
}

/// If the last two elements of `raw_tokens` are both `RawToken::BlankLines`,
/// merge them into one `RawToken::BlankLines`.
///
/// Returns `true` if the last two elements were merged.
///
/// # Panics
/// If `input`'s offset to the second-to-last element of `self.toks` is not
/// 0.
fn merge_last_blanklines<'i>(src: &'i Source, raw_tokens: &mut RawTokens<'i>) -> bool {
    let len = raw_tokens.len();
    if len < 2 {
        return false;
    }
    // If either of the last two elements isn't `BlankLines`, we're done.
    let (prev, last) = match (&raw_tokens[len - 2], &raw_tokens[len - 1]) {
        (RawToken::BlankLines(prev), RawToken::BlankLines(last)) => (prev, last),
        _ => return false,
    };

    // A String containing prev and last.
    let joined_fragment = {
        let mut ret =
            String::with_capacity(prev.span.fragment().len() + last.span.fragment().len());
        ret.push_str(prev.span.fragment());
        ret.push_str(last.span.fragment());
        ret
    };
    let merged = RawToken::BlankLines(BlankLines {
        span: src.alloc_span(joined_fragment, prev.span),
        count: prev.count + last.count,
    });
    raw_tokens.truncate(len - 2);
    raw_tokens.push(merged);
    true
}
