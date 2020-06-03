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

use super::parse_util::{
    is_letter, is_mark, is_number, is_punctuation, is_symbol, many_at_least, take_ident,
    take_inline_space1, take_letter1, take_not_inline_space1, take_number1, take_punctuation1,
    take_symbol1,
};
use super::{Source, Span};

/// A parsed command, consisting of a name and arguments.
#[derive(Clone, Debug, PartialEq)]
pub struct Command<'i> {
    /// The command's name.
    pub name: Span<'i>,
    /// The command's arguments.
    pub args: Vec<Argument<'i>>,
}

impl<'i> Command<'i> {
    /// Create a new `Command` from the given name, with no arguments.
    pub fn from_name(name: Span<'i>) -> Self {
        Self::new(name, Vec::new())
    }

    /// Create a new `Command`.
    pub fn new(name: Span<'i>, args: Vec<Argument<'i>>) -> Self {
        Self { name, args }
    }
}

/// An argument to a command.
#[derive(Clone, Debug, PartialEq)]
pub struct Argument<'i> {
    /// The argument's keyword name, if given.
    pub name: Option<Span<'i>>,
    /// The argument's value.
    pub value: Span<'i>,
}

impl<'i> Argument<'i> {
    /// Create a new `Argument`.
    pub fn new(name: Option<Span<'i>>, value: Span<'i>) -> Self {
        Argument { name, value }
    }

    /// Create a new `Argument` with no explicit name.
    pub fn from_value(value: Span<'i>) -> Self {
        Argument { name: None, value }
    }
}

/// Parse a string with balanced braces.
fn balanced_braces<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    context(
        "balanced braces",
        recognize(many0(alt((
            recognize(none_of("{}\\")),
            // Escaped braces
            recognize(preceded(tag("\\"), one_of("{}"))),
            // Other escapes are passed through literally.
            recognize(pair(tag("\\"), anychar)),
            brace_group,
        )))),
    )(i)
}

/// Recognize a group of braces.
pub fn brace_group<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    delimited(take_char('{'), balanced_braces, cut(take_char('}')))(i)
}

/// Parse a command keyword-argument name.
fn command_kwarg_name<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    recognize(many0(none_of("\\{}$=")))(i)
}

fn command_kwarg_value<'a, E: ParseError<Span<'a>>>(
    i: Span<'a>,
) -> IResult<Span, (Option<char>, Span), E> {
    pair(opt(take_char('=')), balanced_braces)(i)
}

/// Parse a command argument.
fn command_arg<'a, E: ParseError<Span<'a>>>(
    arena: &'a Source,
    i: Span<'a>,
) -> IResult<Span<'a>, Argument<'a>, E> {
    preceded(
        opt(take_inline_space1),
        map(
            delimited(
                take_char('{'),
                cut(pair(command_kwarg_name, command_kwarg_value)),
                cut(take_char('}')),
            ),
            |(name, (eq_tok, val))| Argument {
                name: eq_tok.map(|_| name),
                value: eq_tok.map(|_| val).unwrap_or(arena.alloc_span(
                    {
                        let mut nameval =
                            String::with_capacity(name.fragment().len() + val.fragment().len());
                        nameval.push_str(name.fragment());
                        nameval.push_str(val.fragment());
                        nameval
                    },
                    name,
                )),
            },
        ),
    )(i)
}

/// Parse a command name.
fn command_name<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    preceded(tag("\\"), take_ident)(i)
}

/// Parse a command and at least `mandatory_args` args.
pub fn parse_command<'a, E: ParseError<Span<'a>>>(
    arena: &'a Source,
    mandatory_args: usize,
) -> impl Fn(Span<'a>) -> IResult<Span, Command, E> {
    move |i| {
        context(
            "command",
            map(
                pair(
                    command_name,
                    cut(many_at_least(
                        mandatory_args,
                        complete(|i| command_arg(arena, i)),
                    )),
                ),
                |(name, args)| Command { name, args },
            ),
        )(i)
    }
}

#[cfg(test)]
mod test {
    use nom::{
        combinator::all_consuming,
        error::{make_error, ErrorKind, VerboseError, VerboseErrorKind},
    };

    use claim::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;

    use super::super::test_util::{AssertParse, Input};

    #[test]
    fn test_balanced_braces() {
        let builder = AssertParse::new(balanced_braces).all_consuming(true);

        let balanced = builder.build();

        balanced.assert("xxx");
        balanced.assert("{}");
        balanced.assert("{{{}}}");
        balanced.assert("{{{}}{}}{\\}}");
        balanced.assert("{o{{foo}}}barbaz {}");
        balanced.assert("{ \\{ }");

        AssertParse::new(balanced_braces)
            .ok(Box::new(|input, output| {
                assert_eq!(input.offset(0, " {} "), output)
            }))
            .rest(Box::new(|input, rest| {
                assert_eq!(input.offset(4, "}"), rest)
            }))
            .build()
            .assert(" {} }");

        AssertParse::new(balanced_braces)
            .ok(Box::new(|input, output| {
                assert_eq!(input.offset(0, ""), output)
            }))
            .rest(Box::new(|input, rest| {
                assert_eq!(input.offset(0, "}"), rest)
            }))
            .build()
            .assert("}");
    }

    #[test]
    fn test_command_name() {
        AssertParse::new(command_name)
            .ok(Box::new(|i, name| assert_eq!(i.offset(1, "x"), name)))
            .rest(Box::new(|i, rest| assert_eq!(i.offset(2, " {y}"), rest)))
            .build()
            .assert("\\x {y}");
    }

    #[test]
    fn test_command_arg() {
        let source = Source::new("".into());
        let assert = || AssertParse::new(|i| command_arg(&source, i));

        assert()
            .ok(Box::new(|input, arg| {
                assert_eq!(Argument::from_value(input.offset(2, "y")), arg)
            }))
            .rest(Box::new(|input, rest| {
                assert_eq!(input.offset(4, "{z}"), rest)
            }))
            .build()
            .assert(" {y}{z}");

        assert()
            .ok(Box::new(|input, arg| {
                assert_eq!(
                    Argument::new(Some(input.offset(1, "name ")), input.offset(7, " val")),
                    arg
                )
            }))
            .all_consuming(true)
            .build()
            .assert("{name = val}");

        assert()
            .incomplete(Box::new(|_needed| ()))
            .ok(Box::new(|_, _| panic!("Unexpected Ok")))
            .build()
            .assert("");
    }

    #[test]
    fn test_command() {
        let source = Source::new("".into());
        let parse_command_ = |n| parse_command(&source, n);

        // "At least 0 args" will absorb the 1 arg.
        AssertParse::new(parse_command_(0))
            .ok(Box::new(|i, cmd| {
                assert_eq!(
                    Command {
                        name: i.offset(1, "x"),
                        args: vec![Argument::from_value(i.offset(4, "y")),],
                    },
                    cmd
                )
            }))
            .build()
            .assert("\\x {y}");

        // Here we have 1 arg.
        AssertParse::new(parse_command_(1))
            .ok(Box::new(|i, cmd| {
                assert_eq!(
                    Command {
                        name: i.offset(1, "section"),
                        args: vec![Argument::from_value(i.offset(9, "Whatever")),],
                    },
                    cmd
                )
            }))
            .build()
            .assert("\\section{Whatever}");

        // We don't have 3 arguments
        AssertParse::new(parse_command_(3))
            .ok(Box::new(|_, _| panic!("Unexpected Ok")))
            .err(Box::new(|(_rest, _kind)| ()))
            .build()
            .assert("\\section{Whatever}");
    }
}
