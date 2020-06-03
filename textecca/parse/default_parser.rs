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

use super::parse_util::*;
use super::{parse_command, Command, Parser, Source, Span, Token, Tokens};

/// The default textecca parser.
pub fn default_parser<'i>(
    arena: &'i Source,
    input: Span<'i>,
) -> Result<Tokens<'i>, Box<dyn Error + 'i>> {
    all_consuming(many0(alt((
        map(parse_command(arena, 0), Token::from),
        map(recognize(many1(none_of("\\\r\n"))), Token::from),
        newlines(arena.alloc_spans("par".into())),
    ))))(input)
    .map(|(_remaining, tokens)| tokens)
    .map_err(|e: nom::Err<VerboseError<_>>| e.into())
}

fn newlines<'i, E: ParseError<Span<'i>> + 'i>(
    alloc_span: impl Fn(Span<'i>) -> Span<'i> + 'i,
) -> impl Fn(Span<'i>) -> IResult<Span, Token, E> + 'i {
    map(many1(newline), move |nls| {
        let nl = *nls.last().unwrap();
        if nls.len() == 1 {
            // A single newline is nothing special.
            nl.into()
        } else {
            // Multiple newlines is a paragraph.
            Token::from(Command::from_name(alloc_span(nl)))
        }
    })
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::parse::test_util::Input;
    use crate::parse::{Argument, Command};

    #[test]
    fn parse_simple() {
        let input = Input::new("xxx\n\\cmd{foo} bar");
        assert_eq!(
            vec![
                Token::from(input.offset(0, "xxx")),
                input.offset(3, "\n").into(),
                Command::new(
                    input.offset(5, "cmd"),
                    vec![Argument::from_value(input.offset(9, "foo"))]
                )
                .into(),
                input.offset(13, " bar").into()
            ],
            default_parser(&input.arena, input.span).unwrap()
        );

        let input = Input::new("first.\n\nsecond.");
        assert_eq!(
            vec![
                Token::from(input.offset(0, "first.")),
                Command::from_name(input.arena.alloc_span("par".into(), input.slice(7..7))).into(),
                input.offset(8, "second.").into()
            ],
            default_parser(&input.arena, input.span).unwrap()
        );
    }
}
