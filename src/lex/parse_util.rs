use nom::{
    branch::alt,
    bytes::complete::{tag, take as take_bytes},
    bytes::streaming::{take_while, take_while1},
    character::complete::{anychar, char as take_char, none_of, one_of},
    combinator::{all_consuming, complete, cut, map, not, opt, recognize, rest_len, value, verify},
    error::{context, make_error, ErrorKind, ParseError, VerboseError},
    multi::{many0, many1, many1_count, separated_nonempty_list},
    sequence::{pair, preceded, terminated, tuple},
    IResult, Slice,
};
use nom_locate::{position, LocatedSpan};
use unicode_segmentation::UnicodeSegmentation;

use crate::lex::ucd_general_category;

pub type Span<'input, Extra = ()> = LocatedSpan<&'input str, Extra>;
pub type Error<'input, Extra = ()> = VerboseError<Span<'input, Extra>>;

/// Drops the result of a parser.
pub fn drop_parser<I, O, E, F>(f: F) -> impl Fn(I) -> IResult<I, (), E>
where
    I: Clone,
    E: ParseError<I>,
    F: Fn(I) -> IResult<I, O, E>,
{
    value((), f)
}

/// Succeeds if there's no remaining input, errors otherwise.
pub fn eof<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, (), E> {
    not(take_bytes(1usize))(i)
}

/// True if `c` is of [category] `N`.
///
/// [category]: https://unicode.org/reports/tr44/#General_Category_Values
pub fn is_number(c: char) -> bool {
    ucd_general_category::NUMBER.contains_char(c)
}

/// Takes a string of at least 1 consecutive `N` category codepoints.
pub fn take_number1<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    take_while1(is_number)(i)
}

/// True if `c` is of [category] `P`.
///
/// [category]: https://unicode.org/reports/tr44/#General_Category_Values
pub fn is_punctuation(c: char) -> bool {
    ucd_general_category::PUNCTUATION.contains_char(c)
}

/// Takes a string of at least 1 consecutive `P` category codepoints.
pub fn take_punctuation1<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    take_while1(is_punctuation)(i)
}

/// True if `c` is of [category] `S`.
///
/// [category]: https://unicode.org/reports/tr44/#General_Category_Values
pub fn is_symbol(c: char) -> bool {
    ucd_general_category::SYMBOL.contains_char(c)
}

/// Takes a string of at least 1 consecutive `S` category codepoints.
pub fn take_symbol1<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    take_while1(is_symbol)(i)
}

/// True if `c` is of [category] `Zs`.
///
/// [category]: https://unicode.org/reports/tr44/#General_Category_Values
pub fn is_inline_space(c: char) -> bool {
    ucd_general_category::SPACE_SEPARATOR.contains_char(c)
}

/// Takes a string of at least 1 consecutive `Zs` category codepoints.
pub fn take_inline_space1<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    take_while1(is_inline_space)(i)
}

/// Succeeds if the next character is not whitespace.
pub fn peek_printing_char<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, (), E> {
    not(verify(anychar, |c| is_inline_space(*c)))(i)
}

/// Returns the slice up to the next Unicode word boundary.
pub fn next_word_bound<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    match i.fragment().split_word_bounds().next() {
        Some(chunk) => Ok((i.slice(chunk.len()..), i.slice(..chunk.len()))),
        // TODO: Should this be `Incomplete` instead?
        None => Err(nom::Err::Error(make_error(i, ErrorKind::Eof))),
    }
}

/// Returns the slice up to the next EGC boundary.
pub fn next_egc_bound<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    match i.fragment().grapheme_indices(/* extended = */ true).next() {
        Some((_, chunk)) => Ok((i.slice(chunk.len()..), i.slice(..chunk.len()))),
        // TODO: Should this be `Incomplete` instead?
        None => Err(nom::Err::Error(make_error(i, ErrorKind::Eof))),
    }
}
