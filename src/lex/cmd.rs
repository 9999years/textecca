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

use crate::lex::parse_util::{
    is_letter, is_mark, is_number, is_punctuation, is_symbol, many_at_least, take_ident,
    take_inline_space1, take_letter1, take_not_inline_space1, take_number1, take_punctuation1,
    take_symbol1,
};
use crate::lex::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Command<'i> {
    name: Span<'i>,
    args: Vec<Argument<'i>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Argument<'i> {
    name: Option<Span<'i>>,
    value: Span<'i>,
}

impl<'i> Argument<'i> {
    pub fn new(name: Option<Span<'i>>, value: Span<'i>) -> Self {
        Argument { name, value }
    }

    pub fn from_value(value: Span<'i>) -> Self {
        Argument { name: None, value }
    }
}

/// Parse a string with balanced braces.
fn balanced_braces<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    recognize(many0(alt((
        recognize(none_of("{}\\")),
        recognize(preceded(tag("\\"), one_of("{}"))),
        brace_group,
    ))))(i)
}

/// Recognize a group of braces.
fn brace_group<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    delimited(take_char('{'), balanced_braces, cut(take_char('}')))(i)
}

/// Parse a command keyword-argument name.
fn command_kwarg_name<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    recognize(many0(none_of("\\{}$=")))(i)
}

fn command_kwarg_value<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Option<Span>, E> {
    opt(preceded(take_char('='), balanced_braces))(i)
}

/// Parse a command argument.
fn command_arg<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Argument, E> {
    preceded(
        opt(take_inline_space1),
        map(
            delimited(
                take_char('{'),
                pair(command_kwarg_name, command_kwarg_value),
                cut(take_char('}')),
            ),
            |(name, val)| Argument {
                name: val.map(|_| name),
                value: val.unwrap_or(name),
            },
        ),
    )(i)
}

/// Parse a command name.
fn command_name<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    preceded(tag("\\"), take_ident)(i)
}

/// Parse a command and at least `mandatory_args` args.
pub fn command<'a, E: ParseError<Span<'a>>>(
    mandatory_args: usize,
) -> impl Fn(Span<'a>) -> IResult<Span, Command, E> {
    context(
        "command",
        map(
            pair(
                command_name,
                cut(many_at_least(mandatory_args, complete(command_arg))),
            ),
            |(name, args)| Command { name, args },
        ),
    )
}

#[cfg(test)]
mod test {
    use nom::error::{make_error, ErrorKind, VerboseError, VerboseErrorKind};

    use claim::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;

    use crate::lex::test_util::Input;
    use crate::{assert_destructure, assert_parse_failed, assert_parsed_all, test_parse};

    #[test]
    fn test_balanced_braces() {
        macro_rules! assert_braces_balanced {
            ($input:expr) => {
                let (input, res) = test_parse!(balanced_braces, $input);
                assert_parsed_all!(input, res);
            };
        }

        assert_braces_balanced!("xxx");
        assert_braces_balanced!("{}");
        assert_braces_balanced!("{{{}}}");
        assert_braces_balanced!("{{{}}{}}{\\}}");
        assert_braces_balanced!("{o{{foo}}}barbaz {}");
        assert_braces_balanced!("{ \\{ }");

        let (input, res) = test_parse!(balanced_braces, " {} }");
        assert_eq!(
            Ok((
                input.offset(4, "}"),    // remaining
                input.offset(0, " {} "), // parsed
            )),
            res
        );

        let (input, res) = test_parse!(balanced_braces, "}");
        assert_eq!(
            Ok((
                input.offset(0, "}"), // remaining
                input.offset(0, ""),  // parsed
            )),
            res
        );
    }

    #[test]
    fn test_command_name() {
        let (input, res) = test_parse!(command_name, "\\x {y}");
        assert_destructure! {
            let Ok((rest, name)) = res;
            {
                assert_eq!(input.offset(1, "x"), name);
                assert_eq!(input.offset(2, " {y}"), rest);
            }
        };
    }

    #[test]
    fn test_command_arg() {
        let (input, res) = test_parse!(command_arg, " {y}{z}");
        assert_destructure! {
            let Ok((rest, arg)) = res;
            {
                assert_eq!(Argument::from_value(input.offset(2, "y")), arg);
                assert_eq!(input.offset(4, "{z}"), rest);
            }
        };

        let (input, res) = test_parse!(command_arg, "{name = val}");
        assert_destructure! {
            let Ok((rest, arg)) = res;
            {
                assert_eq!(
                    Argument::new(
                        Some(input.offset(1, "name ")),
                        input.offset(7, " val")
                    ),
                    arg
                );
                assert_eq!(input.eof(), rest);
            }
        };

        let (_input, res) = test_parse!(command_arg, "");
        assert_err!(res);
    }

    #[test]
    fn test_command() {
        // "At least 0 args" will absorb the 1 arg.
        let (input, res) = test_parse!(command(0), "\\x {y}");
        assert_parsed_all!(input, res);
        assert_destructure! {
            let Ok((_, cmd)) = res;
            {
                assert_eq!(
                    Command {
                        name: input.offset(1, "x"),
                        args: vec![Argument::from_value(input.offset(4, "y")),],
                    },
                    cmd
                );
            }
        }

        // Here we have 1 arg.
        let (input, res) = test_parse!(command(1), "\\section{Whatever}");
        assert_parsed_all!(input, res);
        assert_destructure! {
            let Ok((_, cmd)) = res;
            {
                assert_eq!(
                    Command {
                        name: input.offset(1, "section"),
                        args: vec![Argument::from_value(input.offset(9, "Whatever")),],
                    },
                    cmd
                );
            }
        }

        // We don't have 3 arguments
        let (_input, res) = test_parse!(command(3), "\\section{Whatever}");
        assert_err!(res);
    }
}
