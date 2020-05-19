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

use crate::lex::parse_util::{take_inline_space1, take_not_inline_space1};
use crate::lex::Span;

struct Command<'i> {
    name: Span<'i>,
    args: Vec<Argument<'i>>,
}

struct Argument<'i> {
    name: String,
    value: Span<'i>,
}

/// Parse a string with balanced braces.
fn balanced_braces<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    recognize(many0(alt((
        recognize(none_of("{}\\")),
        recognize(pair(tag("\\"), one_of("{}"))),
        brace_group,
    ))))(i)
}

/// Recognize a group of braces.
fn brace_group<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    recognize(tuple((take_char('{'), balanced_braces, take_char('}'))))(i)
}

/// Parse a command argument.
fn command_arg<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    preceded(take_inline_space1, brace_group)(i)
}

/// Parse a command name.
fn command_name<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    preceded(tag("\\"), take_not_inline_space1)(i)
}

#[cfg(test)]
mod test {
    use nom::error::{make_error, ErrorKind, VerboseError, VerboseErrorKind};

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::lex::test_util::Input;

    #[test]
    fn test_balanced_braces() {
        macro_rules! assert_braces_balanced {
            ($input:expr) => {
                let input = Input::new($input);
                assert_eq!(
                    Ok((input.eof(), input.as_span())),
                    all_consuming(balanced_braces::<VerboseError<_>>)(input.as_span()),
                );
            };
        }

        macro_rules! assert_braces_unbalanced {
            ($input:expr, offset $offset:expr, at $fragment:expr) => {
                let input = Input::new($input);
                assert!(input.as_span().fragment()[$offset..].starts_with($fragment));
                assert_eq!(
                    Err(nom::Err::Error(VerboseError {
                        errors: vec![(
                            input.slice($offset..),
                            VerboseErrorKind::Nom(ErrorKind::Eof)
                        )]
                    })),
                    all_consuming(balanced_braces::<VerboseError<_>>)(input.as_span()),
                );
            };
        }

        assert_braces_balanced!("xxx");
        assert_braces_balanced!("{}");
        assert_braces_balanced!("{{{}}}");
        assert_braces_balanced!("{{{}}{}}{\\}}");
        assert_braces_balanced!("{o{{foo}}}barbaz {}");
        assert_braces_balanced!("{ \\{ }");

        assert_braces_unbalanced!(" {} }", offset 4, at "}");
        assert_braces_unbalanced!("{", offset 0, at "{");
    }
}
