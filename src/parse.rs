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
use nom_locate::{position, LocatedSpan};

type Span<'input> = LocatedSpan<&'input str>;

fn drop<'a, I, O, E, F>(f: F) -> impl Fn(I) -> IResult<I, (), E>
where
    I: Clone,
    E: ParseError<I>,
    F: Fn(I) -> IResult<I, O, E>,
{
    map(f, |_| ())
}

/// Succeeds if there's no remaining input, fails otherwise.
fn eof<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, (), E> {
    not(take_bytes(1usize))(i)
}

fn inline_whitespace<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    recognize(many1(one_of(" \t")))(i)
}

fn newline<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    // TODO: Only accept one style of line-ending per-file?
    recognize(pair(opt(inline_whitespace), alt((tag("\n"), tag("\r\n")))))(i)
}

fn not_newline<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, char, E> {
    // \r is only valid for newlines
    none_of("\r\n")(i)
}

fn paragraph_sep<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    context(
        "paragraph separator",
        recognize(verify(many1_count(newline), |newlines| *newlines > 2)),
    )(i)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph<'input> {
    span: Span<'input>,
    content: &'input str,
    sep: Option<&'input str>,
}

fn paragraph<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    context(
        "paragraph",
        recognize(pair(
            separated_nonempty_list(newline, many1(not_newline)),
            alt((eof, drop(paragraph_sep))),
        )),
    )(i)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseTree<'input> {
    paragraphs: Vec<Span<'input>>,
}

pub fn parse<'a, E: ParseError<Span<'a>>>(i: &'a str) -> IResult<Span, ParseTree, E> {
    let i_span = Span::new(i);
    // all_consuming(map(many0(paragraph), |paragraphs| ParseTree { paragraphs }))(i_span)
    map(paragraph, |p| ParseTree {
        paragraphs: vec![p],
    })(i_span)
}
