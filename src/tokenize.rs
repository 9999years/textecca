use nom::{
    branch::alt,
    bytes::complete::{tag, take as take_bytes},
    character::complete::{anychar, char as take_char, none_of, one_of},
    combinator::{
        all_consuming, complete, cut, iterator, map, map_parser, not, opt, peek, recognize,
        rest_len, value, verify,
    },
    error::{context, make_error, ErrorKind, ParseError, VerboseError},
    multi::{
        fold_many0, many0, many0_count, many1, many1_count, many_till, separated_nonempty_list,
    },
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};
use nom_locate::position;
use unicode_segmentation::UnicodeSegmentation;

use crate::parse_util::{
    drop_parser, eof, is_inline_space, is_number, is_punctuation, is_symbol, next_word_bound,
    peek_printing_char, take_inline_space1, take_number1, take_punctuation1, take_symbol1,
};
use crate::Span;

/// A group of one or more blank lines.
/// The lines may include whitespace.
#[derive(Debug, Clone, PartialEq)]
pub struct BlankLines<'i> {
    /// The span encompassing the blank lines, not including the newline that
    /// starts the span of blank lines.
    ///
    /// For example, in the string `"Foo\n\nBar"`, the span would contain only
    /// the second `"\n"`.
    pub span: Span<'i>,

    /// The count of blank lines in the group.
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'i> {
    /// A new level of indentation. The span gives the new additional indentation
    /// prefix, which is added to the previous indentation.
    Indent(Span<'i>),

    /// A decrement of some number of indented blocks.
    Deindent(usize),

    /// A word. This is nebulously defined and will be refined over time.
    ///
    /// By default, word boundaries are computed according to [UAX 29][tr29-wb].
    /// However, some grammars may wish to split words into smaller tokens (for
    /// example, while `don't` would be one word in running text, but `f'g` may
    /// be several tokens in an equation context).
    ///
    /// [tr29-wb]: https://unicode.org/reports/tr29/#Default_Word_Boundaries
    Word(Span<'i>),

    /// Inline space, e.g. between words or at the end of a line.
    Space(Span<'i>),

    /// A group of punctuation or symbol codepoints separated by word/space boundaries.
    ///
    /// Punct contains codepoints of the [categories `P` and
    /// `S`][tr44-categories] (punctuation and symbols).
    ///
    /// [tr44-categories]: https://unicode.org/reports/tr44/#General_Category_Values
    Punct(Span<'i>),

    /// A group of number codepoints ([category `N`][tr44-categories]).
    ///
    /// Note that in many cases, a "number" may contain one or more `Num` tokens
    /// surrounded by `Punct` or `Word` tokens (possible edge cases include
    /// strings like `1 million`, `0x33`, `1,000`, `3.22`).
    ///
    /// [tr44-categories]: https://unicode.org/reports/tr44/#General_Category_Values
    Num(Span<'i>),

    /// A line break.
    Newline(Span<'i>),

    /// One or more blank lines.
    BlankLines(BlankLines<'i>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tokens<'i> {
    pub toks: Vec<Token<'i>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Tokenizer<'i> {
    indent: Vec<&'i str>,
    toks: Vec<Token<'i>>,
}

impl<'i> Tokenizer<'i> {
    fn new() -> Self {
        Default::default()
    }

    fn parse_immediate_newline<E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Span, E> {
        alt((tag("\n"), tag("\r\n")))(i)
    }

    /// Recognizes a newline, optionally preceeded by inline whitespace.
    fn parse_newline<E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Token, E> {
        // TODO: Only accept one style of line-ending per-file?
        // TODO: (Perf) Restrict take_inline_space1 to only tabs/spaces?
        map(
            recognize(pair(opt(take_inline_space1), Self::parse_immediate_newline)),
            Token::Newline,
        )(i)
    }

    fn parse_after_indent<E: ParseError<Span<'i>> + Clone>(
        &mut self,
        i: Span<'i>,
    ) -> IResult<Span<'i>, (), E> {
        let mut it = iterator(
            i,
            pair(
                not(Self::parse_immediate_newline),
                map(next_word_bound, |chunk| {
                    let c = chunk.fragment().chars().next().unwrap();
                    if is_punctuation(c) || is_symbol(c) {
                        Token::Punct(chunk)
                    } else if is_number(c) {
                        Token::Num(chunk)
                    } else if is_inline_space(c) {
                        Token::Space(chunk)
                    } else {
                        Token::Word(chunk)
                    }
                }),
            ),
        );
        for ((), tok) in &mut it {
            self.toks.push(tok);
        }
        it.finish()
    }

    /// Recognizes indentation at the start of a line.
    ///
    /// Returns any of `Token::Indent`, `Token::Deindent`, or `Token::Newline`
    ///
    /// `None` indicates no change in indentation.
    fn parse_indent<E: ParseError<Span<'i>>>(
        &mut self,
        i: Span<'i>,
    ) -> IResult<Span<'i>, Option<Token<'i>>, E> {
        let mut rest = i;
        for (i, chunk) in self.indent.iter().enumerate() {
            println!("Parsing indent chunk number {}, {:#?}", i, chunk);
            dbg!(rest);
            let (next_rest, deindent) = alt((
                // The next chunk of indentation.
                value(None, tag(*chunk)),
                // The next character is *not* whitespace -- deindent.
                map(peek_printing_char, |()| {
                    Some(Token::Deindent(self.indent.len() - i))
                }),
                // The next character *is* whitespace; if we have a newline,
                // that's valid. Otherwise, we have an indentation error.
                context(
                    "blank line or indentation matches no outer block",
                    cut(map(Self::parse_newline, Some)),
                ),
            ))(rest)?;

            if let Some(tok) = &deindent {
                if let Token::Deindent(count) = tok {
                    dbg!(&self.indent);
                    self.indent.truncate(self.indent.len() - count);
                }
                return Ok((next_rest, deindent));
            }

            rest = next_rest;
        }

        alt((
            // The next character is *not* whitespace -- no change in indentation.
            value(None, peek_printing_char),
            // The next character *is* whitespace; if we have a newline,
            // that's a blank line. Otherwise, we have a nested block.
            context(
                "nested block",
                map(
                    pair(take_inline_space1, opt(Self::parse_immediate_newline)),
                    |(indent, maybe_newline)| {
                        Some(maybe_newline.map_or_else(|| Token::Indent(indent), Token::Newline))
                    },
                ),
            ),
        ))(rest)
    }

    fn parse_line<E: ParseError<Span<'i>> + Clone>(
        &mut self,
        i: Span<'i>,
    ) -> IResult<Span<'i>, (), E> {
        let (rest, maybe_tok) = self.parse_indent(i)?;

        if let Some(Token::Newline(span)) = maybe_tok {
            self.toks
                .push(Token::BlankLines(BlankLines { span, count: 1 }));
            return Ok((rest, ()));
        }

        if let Some(tok) = maybe_tok {
            self.toks.push(tok);
        }

        let (rest, ()) = self.parse_after_indent(rest)?;

        let (rest, newline) = alt((Self::parse_immediate_newline, recognize(eof)))(rest)?;

        self.toks.push(Token::Newline(newline));
        Ok((rest, ()))
    }

    fn tokenize<E: ParseError<Span<'i>> + Clone>(
        &mut self,
        input: Span<'i>,
    ) -> IResult<Span<'i>, (), E> {
        // complete(drop_parser(many0_count(|i: Span<'i>| self.parse_line(i))))(input)
        let mut rest = input;
        while !rest.fragment().is_empty() {
            let (next_rest, ()) = self.parse_line(rest)?;
            rest = next_rest;
        }
        Ok((rest, ()))
    }
}

impl<'i> Into<Tokens<'i>> for Tokenizer<'i> {
    fn into(self) -> Tokens<'i> {
        Tokens { toks: self.toks }
    }
}

pub fn tokenize_parser<'i, E: ParseError<Span<'i>> + Clone>(
    input: Span<'i>,
) -> IResult<Span, Tokens, E> {
    let mut tokenizer = Tokenizer::new();
    let (rest, ()) = tokenizer.tokenize(input)?;
    Ok((rest, tokenizer.into()))
}

pub fn tokenize<'i, E: ParseError<Span<'i>> + Clone>(
    input: Span<'i>,
) -> Result<Tokens, nom::Err<E>> {
    tokenize_parser(input).map(|(_, toks)| toks)
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::test_util::Input;

    macro_rules! assert_toks {
        ($input_name:ident, $toks:expr, $input:expr $(,)?) => {
            let $input_name = Input::new($input);
            assert_eq!(
                Ok(Tokens { toks: $toks }),
                tokenize::<VerboseError<_>>($input_name.as_span())
            );
        };
    }

    #[test]
    fn tokenize_simple() {
        assert_toks!(
            input,
            vec![Token::Word(input.slice(0..3)), Token::Newline(input.eof()),],
            "xxx",
        );

        assert_toks!(input, vec![], "",);

        assert_toks!(
            input,
            vec![
                Token::Word(input.slice(0..3)),
                Token::Newline(input.slice(3..)),
            ],
            "xxx\n",
        );

        // assert_toks!(
        //     input,
        //     vec![
        //         Token::Word(input.slice(0..3)),
        //         Token::BlankLines(BlankLines {
        //             span: input.slice(3..),
        //             count: 1,
        //         }),
        //     ],
        //     "xxx\n\n",
        // );
    }

    #[test]
    fn tokenize_indent() {
        assert_toks!(
            input,
            vec![
                Token::Word(input.offset(0, "no_indent")),
                Token::Newline(input.offset(9, "\n")),
                Token::Indent(input.offset(10, "    ")),
                Token::Word(input.offset(14, "indent")),
                Token::Newline(input.offset(20, "\n")),
                Token::Deindent(1),
                Token::Word(input.offset(21, "deindent_1")),
                Token::Newline(input.offset(31, "\n")),
                Token::Word(input.offset(32, "same_indent")),
                Token::Newline(input.offset(43, "\n")),
            ],
            indoc!(
                r#"
                no_indent
                    indent
                deindent_1
                same_indent
                "#
            )
        );

        // This should fail!
        assert_toks!(
            input,
            vec![],
            indoc!(
                r#"
                no_indent
                    extra_indent
                  error
                "#
            )
        );

        assert_toks!(
            input,
            vec![],
            indoc!(
                r#"
                no_indent
                    extra_indent
                        extra_indent
                    deindent_1
                    same_indent
                            extra_indent
                deindent_2
                same_indent
                "#
            )
        );
    }
}
