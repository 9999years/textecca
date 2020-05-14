#![allow(dead_code)] // TODO: Remove when parser works reasonably.

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

type Span<'input, Extra = ()> = LocatedSpan<&'input str, Extra>;

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

fn inline_printing<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    recognize(many1(none_of(" \t\r\n")))(i)
}

fn newline<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    // TODO: Only accept one style of line-ending per-file?
    context(
        "newline",
        recognize(pair(opt(inline_whitespace), alt((tag("\n"), tag("\r\n"))))),
    )(i)
}

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

fn paragraph_sep<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    context(
        "paragraph separator",
        recognize(verify(many1_count(newline), |newlines| *newlines >= 2)),
    )(i)
}

fn lines<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Vec<Span>, E> {
    context("lines", separated_nonempty_list(newline, nonempty_line))(i)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph<'input> {
    content: Span<'input>,
    sep: Span<'input>,
}

fn paragraph<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Paragraph, E> {
    context(
        "paragraph",
        map(
            pair(
                recognize(lines),
                cut(context(
                    "EOF or paragraph separator",
                    alt((recognize(eof), paragraph_sep)),
                )),
            ),
            |(content, sep)| Paragraph { content, sep },
        ),
    )(i)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseTree<'input> {
    paragraphs: Vec<Paragraph<'input>>,
}

pub fn parse<'a, E: ParseError<Span<'a>>>(i: &'a str) -> IResult<Span, ParseTree, E> {
    let i_span = Span::new(i);
    all_consuming(map(many0(paragraph), |paragraphs| ParseTree { paragraphs }))(i_span)
    // map(paragraph, |p| ParseTree {
    //     paragraphs: vec![p],
    // })(i_span)
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

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
    fn parse_paragraph() {
        let input = Input::new("a one-line paragraph");
        let res: IResult<_, _, VerboseError<_>> = parse(input.into());
        assert_eq!(
            ParseTree {
                paragraphs: vec![Paragraph {
                    content: input.as_span(),
                    sep: input.span_eof(1),
                }]
            },
            res.unwrap().1
        );

        let input = Input::new("a paragraph with line-endings\n\n");
        let res: IResult<_, _, VerboseError<_>> = parse(input.into());
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
            res
        );
    }
}
