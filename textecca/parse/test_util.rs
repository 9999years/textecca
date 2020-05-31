use std::fmt;
use std::marker::PhantomData;

use nom::{
    combinator::map,
    error::{ErrorKind, ParseError},
    multi::separated_nonempty_list,
    IResult, InputLength, Slice,
};

use typed_builder::TypedBuilder;

use claim::*;
use pretty_assertions::assert_eq;

use super::{Source, Span};

#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    pub span: Span<'static>,
    pub arena: Source,
}

impl Input {
    pub fn new(input: &'static str) -> Self {
        Self {
            span: Span::new(input),
            arena: Source::new(input.to_owned()),
        }
    }

    /// This fragment as a Span, i.e. with offset 0.
    pub fn as_span(&self) -> Span {
        self.span
    }

    /// The Span at this fragment's end.
    pub fn eof(&self) -> Span {
        self.span.slice(self.span.input_len()..)
    }

    /// Get a span from a given offset and length.
    pub fn offset_len(&self, offset: usize, length: usize) -> Span {
        self.span.slice(offset..offset + length)
    }

    /// Get a span from a given offset and substring.
    pub fn offset(&self, offset: usize, fragment: &'static str) -> Span {
        let bad_offset = offset > self.span.fragment().len()
            || offset + fragment.len() > self.span.fragment().len();
        let ret = if bad_offset {
            None
        } else {
            Some(self.span.slice(offset..offset + fragment.len()))
        };
        if ret.map(|ret| ret.fragment() != &fragment).unwrap_or(true) {
            let probable_index = self
                .span
                .fragment()
                // Find the leftmost occurance of `fragment` after `offset`...
                .match_indices(fragment)
                .find(|(i, _)| i > &offset)
                .or_else(|| {
                    // Or, if no occurance was found, find the last occurance of
                    // `fragment` *before* `offset`.
                    self.span
                        .fragment()
                        .rmatch_indices(fragment)
                        .find(|(i, _)| i < &offset)
                })
                .map(|(i, _)| {
                    format!(
                        "\nI found that fragment at {}..{}. Did you mean that offset?",
                        i,
                        i + fragment.len()
                    )
                })
                .unwrap_or_default();
            panic!(
                "Fragment {fragment:#?} at {start}..{end} {ret}{probable}",
                fragment = fragment,
                start = offset,
                end = offset + fragment.len(),
                ret = ret
                    .map(|ret| format!("doesn't match span: {:#?}", ret.fragment()))
                    .unwrap_or_else(|| format!(
                        "out of bounds starting at index {}",
                        self.span.fragment().len()
                    )),
                probable = probable_index
            );
        }
        ret.unwrap()
    }

    pub fn slice<R>(&self, range: R) -> Span
    where
        Span<'static>: Slice<R>,
    {
        self.span.slice(range)
    }
}

impl Into<&'static str> for Input {
    fn into(self) -> &'static str {
        self.span.fragment()
    }
}

fn assert_parse_err(e: impl fmt::Debug) {
    panic!("Unexpected Err(nom::Err::Failure({:#?})).", e);
}

fn assert_parse_incomplete(needed: nom::Needed) {
    panic!("Unexpected Err(nom::Err::Incomplete({:#?})).", needed);
}

#[derive(TypedBuilder)]
pub struct AssertParse<Parser, O> {
    parser: Parser,

    #[builder(default = false)]
    all_consuming: bool,

    #[builder(default=Box::new(|_i, _output| ()))]
    ok: Box<dyn Fn(&Input, O) -> ()>,

    #[builder(default=Box::new(|err| assert_parse_err(err)))]
    err: Box<dyn Fn((Span<'static>, ErrorKind)) -> ()>,

    #[builder(default=Box::new(|needed| assert_parse_incomplete(needed)))]
    incomplete: Box<dyn Fn(nom::Needed) -> ()>,

    #[builder(default=Box::new(|_i, _rest| ()))]
    rest: Box<dyn Fn(&Input, Span<'static>) -> ()>,
}

impl<Parser, O> AssertParse<Parser, O>
where
    Parser: Fn(Span<'static>) -> IResult<Span<'static>, O, (Span<'static>, ErrorKind)>,
{
    pub fn new(parser: Parser) -> AssertParseBuilder<((Parser,), (), (), (), (), ()), Parser, O> {
        Self::builder().parser(parser)
    }

    pub fn assert(&self, input: &'static str) {
        let input = Input::new(input);
        let res = (self.parser)(input.span);
        match res {
            Ok((rest, output)) => {
                (self.ok)(&input, output);
                (self.rest)(&input, rest);
                if self.all_consuming {
                    assert_eq!(input.eof(), rest);
                }
            }
            Err(err) => match err {
                nom::Err::Incomplete(needed) => (self.incomplete)(needed),
                nom::Err::Error(err) => panic!(
                    "Unexpected nom::Err::Error, expected Incomplete or Failure.\n{:#?}",
                    err
                ),
                nom::Err::Failure(err) => (self.err)(err),
            },
        }
    }
}
