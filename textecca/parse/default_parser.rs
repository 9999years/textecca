use std::error::Error;

use nom::{
    branch::alt,
    bytes::complete::{tag, take as take_bytes},
    bytes::streaming::{take_while, take_while1},
    character::complete::{anychar, char as take_char, none_of, one_of},
    combinator::*,
    error::{make_error, ErrorKind, ParseError, VerboseError},
    multi::*,
    sequence::*,
    IResult, Slice,
};

use super::{
    lex, parse_command, Command, Parser, RawToken, RawTokens, Source, Span, Token, Tokens,
};

/// The default textecca parser.
pub fn default_parser<'i>(
    arena: &'i Source,
    input: Span<'i>,
) -> Result<Tokens<'i>, Box<dyn Error + 'i>> {
    all_consuming(many0(alt((
        map(parse_command(0), Token::from),
        map(recognize(many1(anychar)), Token::from),
    ))))(input)
    .map(|(_remaining, tokens)| tokens)
    .map_err(|e: nom::Err<VerboseError<_>>| e.into())
}
