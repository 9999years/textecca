use nom::{
    combinator::map, error::ParseError, multi::separated_nonempty_list, IResult, InputLength, Slice,
};

use crate::lex::Span;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Input {
    span: Span<'static>,
}

impl Input {
    pub fn new(input: &'static str) -> Self {
        Self {
            span: Span::new(input),
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
        let ret = self.span.slice(offset..offset + fragment.len());
        if ret.fragment() != &fragment {
            panic!(
                "Fragment {} doesn't match span: {}",
                fragment,
                ret.fragment()
            );
        }
        ret
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
