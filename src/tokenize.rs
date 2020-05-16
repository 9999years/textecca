#[allow(unused_imports)]
use nom::{
    branch::alt,
    bytes::complete::{tag, take as take_bytes},
    character::complete::{anychar, char as take_char, none_of, one_of},
    combinator::{
        all_consuming, complete, cut, map, map_parser, not, opt, peek, recognize, rest_len, value,
        verify,
    },
    error::{context, make_error, ErrorKind, ParseError, VerboseError},
    multi::{fold_many0, many0, many1, many1_count, many_till, separated_nonempty_list},
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};
use nom_locate::position;
use unicode_segmentation::UnicodeSegmentation;

use crate::parse_util::{
    eof, is_inline_space, is_number, is_punctuation, is_symbol, next_word_bound,
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

    fn parse_after_indent<E: ParseError<Span<'i>>>(i: Span<'i>) -> IResult<Span, Vec<Token>, E> {
        map(
            many_till(
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
                Self::parse_immediate_newline,
            ),
            |(toks, _)| toks,
        )(i)
    }

    /// Recognizes indentation at the start of a line.
    ///
    /// Returns any of `Token::Indent`, `Token::Deindent`, or `Token::Newline`
    ///
    /// `None` indicates no change in indentation.
    fn parse_indent<E: ParseError<Span<'i>>>(
        &self,
        i: Span<'i>,
    ) -> IResult<Span, Option<Token>, E> {
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

    fn parse_line<E: ParseError<Span<'i>>>(&self, i: Span<'i>) -> IResult<Span, Vec<Token>, E> {
        let (rest, maybe_tok) = self.parse_indent(i)?;

        if let Some(Token::Newline(span)) = maybe_tok {
            return Ok((rest, vec![Token::BlankLines(BlankLines { span, count: 1 })]));
        }

        let mut ret = Vec::new();
        if let Some(tok) = maybe_tok {
            ret.push(tok);
        }

        let (rest, toks) = Self::parse_after_indent(rest)?;

        ret.extend(toks);

        let (rest, newline) = alt((Self::parse_immediate_newline, recognize(eof)))(rest)?;

        ret.push(Token::Newline(newline));
        Ok((rest, ret))
    }

    fn tokenize<'input: 'i, E: ParseError<Span<'i>>>(
        &mut self,
        input: Span<'i>,
    ) -> IResult<Span<'i>, Tokens<'i>, E> {
        map(
            fold_many0(
                |i: Span<'i>| self.parse_line(i),
                Vec::new(),
                |mut acc, toks| {
                    acc.extend(toks);
                    acc
                },
            ),
            |toks| Tokens { toks },
        )(input)
    }

    // fn eof<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span, (), E> {
    //     not(take_bytes(1usize))(i)
    // }
}

pub fn tokenize<'i, E: ParseError<Span<'i>>>(input: Span<'i>) -> IResult<Span, Tokens, E> {
    let mut tokenizer = Tokenizer::new();
    tokenizer.tokenize(input)
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;
}
