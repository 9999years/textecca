use itertools::Itertools;
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
    IResult, Offset, Slice,
};
use nom_locate::position;
use unicode_segmentation::UnicodeSegmentation;

use crate::lex::parse_util::{
    drop_parser, eof, is_inline_space, is_number, is_punctuation, is_symbol, next_egc_bound,
    peek_printing_char, take_inline_space1, take_number1, take_punctuation1, take_symbol1,
};
use crate::lex::Span;

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
        #[derive(Clone, Copy, PartialEq, Debug)]
        enum TokenType {
            Punct,
            Num,
            Space,
            Word,
        }

        impl From<Span<'_>> for TokenType {
            fn from(span: Span) -> Self {
                let c = span.fragment().chars().next().unwrap();
                if is_punctuation(c) || is_symbol(c) {
                    TokenType::Punct
                } else if is_number(c) {
                    TokenType::Num
                } else if is_inline_space(c) {
                    TokenType::Space
                } else {
                    TokenType::Word
                }
            }
        }

        let mut push_current_tok = |ty, ofs, len: usize| {
            let span = i.slice(ofs..ofs + len);
            self.toks.push(match ty {
                TokenType::Punct => Token::Punct(span),
                TokenType::Num => Token::Num(span),
                TokenType::Space => Token::Space(span),
                TokenType::Word => Token::Word(span),
            });
        };

        let mut starting_offset = 0;

        let mut it = iterator(
            i,
            preceded(not(Self::parse_immediate_newline), next_egc_bound),
        );
        for (chunk_type, chunk) in &it.group_by(|egc| TokenType::from(*egc)) {
            let chunk_len = chunk.map(|egc| egc.fragment().len()).sum();
            push_current_tok(chunk_type, starting_offset, chunk_len);
            starting_offset += chunk_len;
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

            if deindent.is_some() {
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
            match tok {
                Token::Indent(span) => {
                    self.indent.push(span.fragment());
                }
                Token::Deindent(count) => {
                    self.indent.truncate(self.indent.len() - count);
                }
                _ => {
                    unreachable!();
                }
            }
            self.toks.push(tok);
        }

        let (rest, ()) = self.parse_after_indent(rest)?;

        let blank_line = rest.location_offset() == i.location_offset();

        let (rest, newline) = alt((Self::parse_immediate_newline, recognize(eof)))(rest)?;

        self.toks.push(if blank_line {
            Token::BlankLines(BlankLines {
                span: newline,
                count: 1,
            })
        } else {
            Token::Newline(newline)
        });
        Ok((rest, ()))
    }

    /// If the last two elements of `self.toks` are both `Token::BlankLines`,
    /// merge them into one `Token::BlankLines` using `input`.
    ///
    /// # Panics
    /// If `input`'s offset to the second-to-last element of `self.toks` is not
    /// 0.
    fn merge_last_blanklines(&mut self, input: &Span<'i>) -> bool {
        let len = self.toks.len();
        let prev = match self.toks.get(len - 2) {
            Some(Token::BlankLines(blanklines)) => blanklines,
            _ => return false,
        };
        let last = match self.toks.get(len - 1) {
            Some(Token::BlankLines(blanklines)) => blanklines,
            _ => return false,
        };

        if input.offset(&prev.span) != 0 {
            panic!(
                "input = {} should have offset 0 to prev = {}.",
                input, prev.span
            );
        }

        let merged = Token::BlankLines(BlankLines {
            span: input.slice(..prev.span.fragment().len() + last.span.fragment().len()),
            count: prev.count + last.count,
        });
        self.toks.truncate(self.toks.len() - 2);
        self.toks.push(merged);
        true
    }

    fn tokenize<E: ParseError<Span<'i>> + Clone>(
        &mut self,
        input: Span<'i>,
    ) -> IResult<Span<'i>, (), E> {
        let mut rest = input;
        let mut prev_rest = input;
        while !rest.fragment().is_empty() {
            let (next_rest, ()) = self.parse_line(rest)?;
            if !self.merge_last_blanklines(&prev_rest) {
                // If we *didn't* merge the last two elements of `self.toks`,
                // the remaining input after the *previous* iteration of this
                // loop *will* be different in the next iteration.
                // (Confusing, I know...)
                prev_rest = rest;
            }
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
    use crate::lex::test_util::Input;

    macro_rules! assert_toks {
        ($input_name:ident, $toks:expr, $input:expr $(,)?) => {
            let $input_name = Input::new($input);
            assert_eq!(
                Ok(Tokens { toks: $toks }),
                tokenize::<VerboseError<_>>($input_name.as_span())
            );
        };
    }

    macro_rules! assert_toks_err {
        ($input:expr $(,)?) => {
            let input = Input::new($input);
            assert!(tokenize::<VerboseError<_>>(input.as_span()).is_err());
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
    }

    #[test]
    fn blanklines() {
        assert_toks!(
            input,
            vec![
                Token::Word(input.offset(0, "foo")),
                Token::Newline(input.offset(3, "\n")),
                Token::BlankLines(BlankLines {
                    span: input.offset(4, "\n"),
                    count: 1
                }),
                Token::Word(input.offset(5, "bar")),
                Token::Newline(input.offset(8, "")),
            ],
            "foo\n\nbar",
        );

        assert_toks!(
            input,
            vec![
                Token::Punct(input.offset(0, "|||")),
                Token::Newline(input.offset(3, "\n")),
                Token::BlankLines(BlankLines {
                    span: input.offset(4, "\n\n\n\n\n"),
                    count: 5
                }),
                Token::Punct(input.offset(9, "|||")),
                Token::Newline(input.offset(12, "")),
            ],
            "|||\n\n\n\n\n\n|||",
        );
    }

    #[test]
    fn tokenize_indent() {
        assert_toks!(
            input,
            vec![
                Token::Word(input.offset(0, "no")),
                Token::Punct(input.offset(2, "_")),
                Token::Word(input.offset(3, "indent")),
                Token::Newline(input.offset(9, "\n")),
                Token::Indent(input.offset(10, "    ")),
                Token::Word(input.offset(14, "indent")),
                Token::Newline(input.offset(20, "\n")),
                Token::Deindent(1),
                Token::Word(input.offset(21, "deindent")),
                Token::Punct(input.offset(29, "_")),
                Token::Num(input.offset(30, "1")),
                Token::Newline(input.offset(31, "\n")),
                Token::Word(input.offset(32, "same")),
                Token::Punct(input.offset(36, "_")),
                Token::Word(input.offset(37, "indent")),
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

        // Indentation error!
        assert_toks_err!(indoc!(
            r#"
                no_indent
                    extra_indent
                  error
                "#
        ));

        assert_toks!(
            input,
            vec![
                Token::Word(input.offset(0, "no",)),
                Token::Punct(input.offset(2, "_",)),
                Token::Word(input.offset(3, "indent",)),
                Token::Newline(input.offset(9, "\n",)),
                Token::Indent(input.offset(10, "    ",)),
                Token::Word(input.offset(14, "extra",)),
                Token::Punct(input.offset(19, "_",)),
                Token::Word(input.offset(20, "indent",)),
                Token::Newline(input.offset(26, "\n",)),
                Token::Indent(input.offset(31, "    ",)),
                Token::Word(input.offset(35, "extra",)),
                Token::Punct(input.offset(40, "_",)),
                Token::Word(input.offset(41, "indent",)),
                Token::Newline(input.offset(47, "\n",)),
                Token::Deindent(1),
                Token::Word(input.offset(52, "deindent",)),
                Token::Punct(input.offset(60, "_",)),
                Token::Num(input.offset(61, "1",)),
                Token::Newline(input.offset(62, "\n",)),
                Token::Word(input.offset(67, "same",)),
                Token::Punct(input.offset(71, "_",)),
                Token::Word(input.offset(72, "indent",)),
                Token::Newline(input.offset(78, "\n",)),
                Token::Indent(input.offset(83, "        ",)),
                Token::Word(input.offset(91, "extra",)),
                Token::Punct(input.offset(96, "_",)),
                Token::Word(input.offset(97, "indent",)),
                Token::Newline(input.offset(103, "\n",)),
                Token::Deindent(2),
                Token::Word(input.offset(104, "deindent",)),
                Token::Space(input.offset(112, " ",)),
                Token::Num(input.offset(113, "2",)),
                Token::Newline(input.offset(114, "\n",)),
                Token::Word(input.offset(115, "same",)),
                Token::Punct(input.offset(119, "_",)),
                Token::Word(input.offset(120, "indent",)),
                Token::Newline(input.offset(126, "\n",)),
            ],
            indoc!(
                r#"
                no_indent
                    extra_indent
                        extra_indent
                    deindent_1
                    same_indent
                            extra_indent
                deindent 2
                same_indent
                "#
            )
        );
    }

    #[test]
    fn tokenize_words() {
        assert_toks!(
            input,
            vec![
                Token::Word(input.offset(0, "this")),
                Token::Space(input.offset(4, " ")),
                Token::Word(input.offset(5, "string")),
                Token::Punct(input.offset(11, "'")),
                Token::Word(input.offset(12, "s")),
                Token::Space(input.offset(13, " ")),
                Token::Word(input.offset(14, "gonna")),
                Token::Space(input.offset(19, " ")),
                Token::Word(input.offset(20, "be")),
                Token::Space(input.offset(22, " ")),
                Token::Word(input.offset(23, "split")),
                Token::Space(input.offset(28, " ")),
                Token::Word(input.offset(29, "in")),
                Token::Num(input.offset(31, "2")),
                Token::Space(input.offset(32, " ")),
                Token::Word(input.offset(33, "several")),
                Token::Punct(input.offset(40, "-")),
                Token::Word(input.offset(41, "different")),
                Token::Punct(input.offset(50, "-")),
                Token::Word(input.offset(51, "tokens")),
                Token::Newline(input.offset(57, "\n")),
            ],
            "this string's gonna be split in2 several-different-tokens\n",
        );

        assert_toks!(
            input,
            vec![
                Token::Num(input.offset(0, "1")),
                Token::Punct(input.offset(1, ",")),
                Token::Num(input.offset(2, "000")),
                Token::Punct(input.offset(5, ",")),
                Token::Num(input.offset(6, "000")),
                Token::Space(input.offset(9, " ")),
                Token::Num(input.offset(10, "9")),
                Token::Punct(input.offset(11, "_")),
                Token::Num(input.offset(12, "876")),
                Token::Punct(input.offset(15, "_")),
                Token::Num(input.offset(16, "543")),
                Token::Space(input.offset(19, " ")),
                Token::Num(input.offset(20, "20")),
                Token::Punct(input.offset(22, ".")),
                Token::Num(input.offset(23, "34")),
                Token::Newline(input.offset(25, "\n")),
            ],
            "1,000,000 9_876_543 20.34\n",
        );
    }
}
