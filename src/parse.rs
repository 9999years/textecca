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

pub type Span<'input, Extra = ()> = LocatedSpan<&'input str, Extra>;
pub type Error<'input, Extra = ()> = VerboseError<Span<'input, Extra>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph<'input> {
    pub content: Span<'input>,
    pub sep: Span<'input>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseTree<'input> {
    pub paragraphs: Vec<Paragraph<'input>>,
}

/// An element within a `Block`; either a child block or a stretch of text.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockElem<'input> {
    Text(Span<'input>),
    Child(Block<'input>),
}

/// A block indented to a particular level.
#[derive(Debug, Clone, PartialEq)]
pub struct Block<'input> {
    /// This block's starting position.
    pub position: Span<'input>,

    /// This block's indent. Each element in the vector represents one nested
    /// block.
    ///
    /// Since we need to parse the entire indent every line, we duplicate this
    /// information in each block.
    ///
    /// `indent` will be empty if and only if this block is top-level.
    pub indent: Vec<&'input str>,

    /// This block's children.
    pub children: Vec<BlockElem<'input>>,
}

/// A change in indentation from a given block.
enum IndentChange<'input> {
    /// Extra indentation found; indicates a nested block.
    More(Span<'input>),

    /// Less indentation found, corresponding to an outer block. The integer
    /// indicates the number of blocks closed.
    Less(u32),

    /// Less indentation found, not corresponding to any outer block; an error
    /// condition. The Span is the indentation found.
    Err(Span<'input>),

    /// No change in indentation found; indicates text in the current block.
    None,
}

impl<'i> Block<'i> {
    /// Recognizes this block's indent at the start of a line.
    fn parse_indent<E: ParseError<Span<'i>>>(&self, i: Span<'i>) -> IResult<Span, IndentChange, E> {
        let mut rest = i;
        for chunk in &self.indent {
            rest = tag(*chunk)(rest)?.0;
        }
        Ok((rest, IndentChange::None))
    }
}

#[derive(Debug, Clone)]
struct BlockParser<'input> {
    indent: &'input str,
}

impl<'i> BlockParser<'i> {}

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

/// Recognizes a non-empty span of inline whitespace, i.e. tabs and spaces.
fn indent<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, Span, E> {
    recognize(pair(many0(take_char('\t')), many0(take_char(' '))))(i)
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
    use crate::test_util::Input;

    fn lines_count<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, usize, E> {
        map(separated_nonempty_list(newline, nonempty_line), |lines| {
            lines.len()
        })(i)
    }

    #[test]
    fn parse_empty() {
        let input = Input::new("");
        assert_eq!(
            Ok((input.eof(), ParseTree { paragraphs: vec![] })),
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
                    sep: input.eof(),
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
                input.eof(),
                ParseTree {
                    paragraphs: vec![Paragraph {
                        content: input.offset_len(0, 29),
                        sep: input.offset_len(29, 2),
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
                input.eof(),
                ParseTree {
                    paragraphs: vec![
                        Paragraph {
                            content: input.offset_len(0, 51),
                            sep: input.offset_len(51, 2),
                        },
                        Paragraph {
                            content: input.offset_len(53, 21),
                            sep: input.offset_len(74, 3),
                        },
                        Paragraph {
                            content: input.offset_len(77, 40),
                            sep: input.offset_len(117, 2),
                        },
                        Paragraph {
                            content: input.offset_len(119, 27),
                            sep: input.offset_len(146, 1),
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
                input.eof(),
                ParseTree {
                    paragraphs: vec![
                        Paragraph {
                            content: input.offset_len(0, 35),
                            sep: input.offset_len(35, 26),
                        },
                        Paragraph {
                            content: input.offset_len(61, 91),
                            sep: input.offset_len(152, 1),
                        }
                    ]
                }
            )),
            parse::<'_, VerboseError<_>>(input.into())
        );
    }
}
