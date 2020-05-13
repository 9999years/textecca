use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char as take_char, none_of, one_of},
    combinator::{all_consuming, map, not, recognize, rest_len, verify},
    multi::{many0, many1, many1_count, separated_nonempty_list},
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};

/// Succeeds if there's no remaining input, fails otherwise.
fn eof(i: &str) -> IResult<&str, &str> {
    recognize(verify(rest_len, |len| *len == 0))(i)
}

fn inline_whitespace(i: &str) -> IResult<&str, &str> {
    recognize(many1(one_of(" \t")))(i)
}

fn newline(i: &str) -> IResult<&str, &str> {
    alt((tag("\n"), tag("\r\n")))(i)
}

fn not_newline(i: &str) -> IResult<&str, char> {
    // \r is only valid for newlines
    none_of("\r\n")(i)
}

fn paragraph_sep(i: &str) -> IResult<&str, usize> {
    verify(many1_count(newline), |newlines| *newlines > 2)(i)
}

fn paragraph(i: &str) -> IResult<&str, &str> {
    recognize(pair(
        separated_nonempty_list(newline, many1(not_newline)),
        alt((recognize(paragraph_sep), eof)),
    ))(i)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseTree {
    paragraphs: Vec<String>,
}

pub fn parse(i: &str) -> IResult<&str, ParseTree> {
    all_consuming(map(many0(paragraph), |paragraphs| ParseTree {
        paragraphs: paragraphs.into_iter().map(String::from).collect(),
    }))(i)
}
