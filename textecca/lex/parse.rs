use nom::{
    branch::alt,
    bytes::complete::{tag, take as take_bytes},
    bytes::streaming::{take_while, take_while1},
    character::complete::{anychar, char as take_char, none_of, one_of},
    combinator::{all_consuming, complete, cut, map, not, opt, recognize, rest_len, value, verify},
    error::{context, make_error, ErrorKind, ParseError, VerboseError},
    multi::{many0, many1, many1_count, separated_nonempty_list},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Slice,
};

use crate::lex::cmd::{command, Command};
use crate::lex::Span;

enum Token<'i> {
    Text(Span<'i>),
    Command(Command<'i>),
}

///
fn parse<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Token, E> {
    alt((
        map(
            recognize(many0(alt((
                none_of("\\{}"),
                preceded(take_char('\\'), one_of("{}")),
            )))),
            Token::Text,
        ),
        map(command(0), Token::Command),
    ))(i)
}
