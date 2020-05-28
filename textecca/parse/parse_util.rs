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
use unicode_segmentation::UnicodeSegmentation;

use super::ucd_tables::{general_category, property_bool};
use super::Span;

/// Drops the result of a parser.
pub fn drop_parser<I, O, E, F>(f: F) -> impl Fn(I) -> IResult<I, (), E>
where
    I: Clone,
    E: ParseError<I>,
    F: Fn(I) -> IResult<I, O, E>,
{
    value((), f)
}

/// Repeats the embedded parser until it fails. Fails if the embedded parser does
/// not succeed at least `n` times.
pub fn many_at_least<I, O, E, F>(n: usize, f: F) -> impl Fn(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + PartialEq,
    E: ParseError<I>,
    F: Fn(I) -> IResult<I, O, E>,
{
    verify(many0(f), move |o: &[O]| o.len() >= n)
}

/// Succeeds if there's no remaining input, errors otherwise.
pub fn eof<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, (), E> {
    not(take_bytes(1usize))(i)
}

/// True if `c` is of [category] `N`.
///
/// [category]: https://unicode.org/reports/tr44/#General_Category_Values
pub fn is_number(c: char) -> bool {
    general_category::NUMBER.contains_char(c)
}

/// Takes a string of at least 1 consecutive `N` category codepoints.
pub fn take_number1<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    take_while1(is_number)(i)
}

/// True if `c` is of [category] `P`.
///
/// [category]: https://unicode.org/reports/tr44/#General_Category_Values
pub fn is_punctuation(c: char) -> bool {
    general_category::PUNCTUATION.contains_char(c)
}

/// Takes a string of at least 1 consecutive `P` category codepoints.
pub fn take_punctuation1<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    take_while1(is_punctuation)(i)
}

/// True if `c` is of [category] `S`.
///
/// [category]: https://unicode.org/reports/tr44/#General_Category_Values
pub fn is_symbol(c: char) -> bool {
    general_category::SYMBOL.contains_char(c)
}

/// Takes a string of at least 1 consecutive `S` category codepoints.
pub fn take_symbol1<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    take_while1(is_symbol)(i)
}

/// True if `c` is of [category] `L`.
///
/// [category]: https://unicode.org/reports/tr44/#General_Category_Values
pub fn is_letter(c: char) -> bool {
    general_category::LETTER.contains_char(c)
}

/// Takes a string of at least 1 consecutive `S` category codepoints.
pub fn take_letter1<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    take_while1(is_letter)(i)
}

/// True if `c` is of [category] `M`.
///
/// [category]: https://unicode.org/reports/tr44/#General_Category_Values
pub fn is_mark(c: char) -> bool {
    general_category::MARK.contains_char(c)
}

/// Takes a string of at least 1 consecutive `S` category codepoints.
pub fn take_mark1<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    take_while1(is_mark)(i)
}

/// True if `c` is of [category] `Zs` or a tab (`'\t'`).
///
/// [category]: https://unicode.org/reports/tr44/#General_Category_Values
pub fn is_inline_space(c: char) -> bool {
    c == ' ' || c == '\t' || general_category::SPACE_SEPARATOR.contains_char(c)
}

/// Takes a string of at least 1 consecutive `Zs` category codepoints.
pub fn take_inline_space1<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    take_while1(is_inline_space)(i)
}

/// Takes a string of at least 1 consecutive non-`Zs` category codepoints.
pub fn take_not_inline_space1<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    take_while1(|c| !is_inline_space(c))(i)
}

/// Succeeds if the next character is not whitespace.
pub fn peek_printing_char<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, (), E> {
    not(verify(anychar, |c| is_inline_space(*c)))(i)
}

/// True if `c` has [property] `XID_Start`.
///
/// [property]: https://unicode.org/reports/tr31/#Default_Identifier_Syntax
pub fn is_xid_start(c: char) -> bool {
    property_bool::XID_START.contains_char(c)
}

/// True if `c` has [property] `XID_Continue`.
///
/// [property]: https://unicode.org/reports/tr31/#Default_Identifier_Syntax
pub fn is_xid_continue(c: char) -> bool {
    property_bool::XID_CONTINUE.contains_char(c)
}

/// Takes a string of 1 `XID_Start` codepoint followed by any number of
/// `XID_Continue` codepoints.
pub fn take_xid<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    recognize(pair(
        verify(anychar, |c| is_xid_start(*c)),
        take_while(is_xid_continue),
    ))(i)
}

/// True if `c` is a valid first character of an identifier.
pub fn is_ident_start(c: char) -> bool {
    is_xid_start(c)
}

/// True if `c` is a valid second or later character of an identifier.
pub fn is_ident_continue(c: char) -> bool {
    is_xid_continue(c)
        || (is_symbol(c) && !"=|$".contains(c))
        || "-".contains(c)
        || (general_category::OTHER_PUNCTUATION.contains_char(c) && !"\"',\\%".contains(c))
}

/// Takes a string of 1 codepoint matching `is_ident_start` followed by any
/// number of codepoints matching `is_ident_continue`.
pub fn take_ident<'i, E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
    recognize(pair(
        verify(anychar, |c| is_ident_start(*c)),
        take_while(is_ident_continue),
    ))(i)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_inline_space() {
        assert!(is_inline_space(' '));
        assert!(is_inline_space('\t'));
        assert!(is_inline_space('\u{a0}')); // nbsp; should this count...?
        assert!(!is_inline_space('\r'));
        assert!(!is_inline_space('\n'));
        assert!(!is_inline_space('X'));
    }

    #[test]
    fn test_symbol() {
        assert!(is_symbol('$'));
        assert!(is_symbol('^'));
        assert!(is_symbol('+'));
        assert!(is_symbol('©'));
        assert!(is_symbol('⇑'));
        assert!(!is_symbol('{'));
        assert!(!is_symbol('}'));
        assert!(!is_symbol('x'));
        assert!(!is_symbol('9'));
        assert!(!is_symbol(' '));
        assert!(!is_symbol('\t'));
        assert!(!is_symbol('\n'));
    }

    #[test]
    fn test_punctuation() {
        assert!(is_punctuation('-'));
        assert!(is_punctuation('*'));
        assert!(is_punctuation('{'));
        assert!(is_punctuation('}'));
        assert!(!is_punctuation('$'));
        assert!(!is_punctuation('^'));
        assert!(!is_punctuation('+'));
        assert!(!is_punctuation('©'));
        assert!(!is_punctuation('⇑'));
    }

    #[test]
    fn test_letter() {
        assert!(is_letter('a'));
        assert!(is_letter('ß'));
        assert!(is_letter('ѷ'));
        assert!(is_letter('画'));
    }

    #[test]
    fn test_number() {
        assert!(is_number('1'));
        assert!(is_number('ⅳ')); // roman numeral
        assert!(is_number('½'));
        assert!(is_number('᭓')); // balinese digit three
        assert!(!is_number(' '));
        assert!(!is_number('x'));
        assert!(!is_number('\t'));
        assert!(!is_number('-'));
    }

    #[test]
    fn test_mark() {
        assert!(is_mark('\u{093B}')); // Devanagari Vowel Sign Ooe
        assert!(is_mark('\u{0489}')); // Combining Cryillic Millions Sign
        assert!(is_mark('\u{0308}')); // Combining Diaeresis
        assert!(!is_mark(' '));
        assert!(!is_mark('\t'));
        assert!(!is_mark('x'));
        assert!(!is_mark('9'));
        assert!(!is_mark('*'));
        assert!(!is_mark('+'));
    }
}
