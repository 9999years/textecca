#![allow(dead_code)] // TODO: Remove when parser works reasonably.

#[allow(unused_imports)]
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
#[allow(unused_imports)]
use nom_locate::{position, LocatedSpan};

pub type Span<'input, Extra = ()> = LocatedSpan<&'input str, Extra>;
pub type Error<'input, Extra = ()> = VerboseError<Span<'input, Extra>>;

/// Drops the result of a parser.
fn drop<I, O, E, F>(f: F) -> impl Fn(I) -> IResult<I, (), E>
where
    I: Clone,
    E: ParseError<I>,
    F: Fn(I) -> IResult<I, O, E>,
{
    map(f, |_| ())
}

/// Succeeds if there's no remaining input, errors otherwise.
fn eof<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, (), E> {
    not(take_bytes(1usize))(i)
}

/// Recognizes a non-empty span of inline whitespace, i.e. tabs and spaces.
fn inline_whitespace<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    recognize(many1(one_of(" \t")))(i)
}

/// Recognizes a non-empty span of inline printing characters, i.e. anything
/// except tabs, spaces, and newlines.
fn inline_printing<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    recognize(many1(none_of(" \t\r\n")))(i)
}

/// Recognizes a newline, optionally preceeded by inline whitespace.
fn newline<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    // TODO: Only accept one style of line-ending per-file?
    context(
        "newline",
        recognize(pair(opt(inline_whitespace), alt((tag("\n"), tag("\r\n"))))),
    )(i)
}

/// Recognizes a non-empty line, i.e. containing at least one printing character.
/// Does not consume the line's trailing newline.
fn nonempty_line<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    // \r is only valid for newlines
    // none_of("\r\n")(i)
    context(
        "nonempty line",
        recognize(tuple((
            opt(inline_whitespace),
            separated_nonempty_list(inline_whitespace, inline_printing),
            opt(inline_whitespace),
        ))),
    )(i)
}

/// Recognizes a sequence of nonempty lines.
fn nonempty_lines<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Vec<Span>, E> {
    context("lines", separated_nonempty_list(newline, nonempty_line))(i)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph<'input> {
    pub content: Span<'input>,
    pub sep: Span<'input>,
}

/// Recognizes a separator between paragraphs, which is *either*:
/// - Any sequence of one or more blank lines. Note that blank lines may include inline whitespace.
/// - Any amount of whitespace, followed by the end of input.
fn paragraph_sep<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    context(
        "paragraph separator or EOF",
        alt((
            recognize(pair(
                newline,
                alt((many1(newline), terminated(many0(newline), eof))),
            )),
            recognize(eof),
        )),
    )(i)
}

/// Recognizes a paragraph (i.e. `nonempty_lines`) followed by either one or more
/// blank lines or the end of input.
fn paragraph<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Paragraph, E> {
    context(
        "paragraph",
        map(
            pair(recognize(nonempty_lines), cut(paragraph_sep)),
            |(content, sep)| Paragraph { content, sep },
        ),
    )(i)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseTree<'input> {
    pub paragraphs: Vec<Paragraph<'input>>,
}

/// Parses the given string as textecca code.
///
/// TODO: Accept other types of input, e.g. from streaming sources.
pub fn parse<'a, E: ParseError<Span<'a>>>(i: &'a str) -> IResult<Span, ParseTree, E> {
    let i_span = Span::new(i);
    all_consuming(map(many0(paragraph), |paragraphs| ParseTree { paragraphs }))(i_span)
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use wyz::Conv;

    use super::*;

    fn lines_count<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, usize, E> {
        map(separated_nonempty_list(newline, nonempty_line), |lines| {
            lines.len()
        })(i)
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Input {
        input: &'static str,
    }

    impl Input {
        fn new(input: &'static str) -> Self {
            Self { input }
        }

        /// This fragment as a Span, i.e. with offset 0.
        fn as_span(&self) -> Span {
            self.span(0, self.input.len(), 1)
        }

        /// The Span at this fragment's end.
        fn span_eof(&self, line: u32) -> Span {
            unsafe { Span::new_from_raw_offset(self.input.len(), line, "", ()) }
        }

        /// Get a span from a given offset and line number, for testing purposes.
        /// Panics if `offset` is not a valid index into `self.input`.
        fn span(&self, offset: usize, length: usize, line: u32) -> Span {
            if offset >= self.input.len() {
                panic!("Bad offset into Input; for EOF use span_eof.");
            }
            // Safety: We've verified offset is a valid index.
            unsafe {
                Span::new_from_raw_offset(offset, line, &self.input[offset..offset + length], ())
            }
        }
    }

    impl Into<&'static str> for Input {
        fn into(self) -> &'static str {
            self.input
        }
    }

    #[test]
    fn parse_empty() {
        let input = Input::new("");
        assert_eq!(
            Ok((input.span_eof(1), ParseTree { paragraphs: vec![] })),
            parse::<'_, VerboseError<_>>(input.into())
        );
    }

    #[test]
    fn parse_single_paragraph_no_sep() {
        let input = Input::new("a one-line paragraph");
        assert_eq!(
            ParseTree {
                paragraphs: vec![Paragraph {
                    content: input.as_span(),
                    sep: input.span_eof(1),
                }]
            },
            parse::<'_, VerboseError<_>>(input.into()).unwrap().1
        );
    }

    #[test]
    fn parse_single_paragraph_and_sep() {
        let input = Input::new("a paragraph with line-endings\n\n");
        assert_eq!(
            Ok((
                input.span_eof(3),
                ParseTree {
                    paragraphs: vec![Paragraph {
                        content: input.span(0, 29, 1),
                        sep: input.span(29, 2, 1),
                    }]
                }
            )),
            parse::<'_, VerboseError<_>>(input.into())
        );
    }

    #[test]
    fn parse_paragraphs() {
        // Multiple paragraphs, multiple blank lines.
        let input = Input::new(indoc!(
            r"
            The first paragraph, which contains
            multiple lines.

            The second paragraph.


            Multiple blank lines between paragraphs.

            Fourth and final paragraph.
            "
        ));
        println!("{:#?}", input.conv::<&str>());
        assert_eq!(
            Ok((
                input.span_eof(10),
                ParseTree {
                    paragraphs: vec![
                        Paragraph {
                            content: input.span(0, 51, 1),
                            sep: input.span(51, 2, 2),
                        },
                        Paragraph {
                            content: input.span(53, 21, 4),
                            sep: input.span(74, 3, 4),
                        },
                        Paragraph {
                            content: input.span(77, 40, 7),
                            sep: input.span(117, 2, 7),
                        },
                        Paragraph {
                            content: input.span(119, 27, 9),
                            sep: input.span(146, 1, 9),
                        },
                    ]
                }
            )),
            parse::<'_, VerboseError<_>>(input.into())
        );
    }

    #[test]
    fn parse_blank_lines() {
        let input = Input::new(include_str!(
            "../test-data/paragraphs/trailing-whitespace.txt"
        ));
        assert_eq!(
            Ok((
                input.span_eof(10),
                ParseTree {
                    paragraphs: vec![
                        Paragraph {
                            content: input.span(0, 35, 1),
                            sep: input.span(35, 26, 1),
                        },
                        Paragraph {
                            content: input.span(61, 91, 8),
                            sep: input.span(152, 1, 9),
                        }
                    ]
                }
            )),
            parse::<'_, VerboseError<_>>(input.into())
        );
    }
}
