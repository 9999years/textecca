use nom::{
    combinator::map,
    error::{ErrorKind, ParseError},
    multi::separated_nonempty_list,
    IResult, InputLength, Slice,
};

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
        let ret = self.span.slice(offset..offset + fragment.len());
        if ret.fragment() != &fragment {
            panic!(
                "Fragment {:#?} doesn't match span: {:#?}",
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

#[macro_export]
macro_rules! test_parse {
    ($parse:expr, $input:expr) => {{
        let input = Input::new($input);
        let parsed: IResult<_, _, (_, ErrorKind)> = $parse(input.span);
        (input, parsed)
    }};
    ($parse:expr, $input:expr,) => {
        parase!($parse, $input);
    };
}

#[macro_export]
macro_rules! assert_parsed_all {
    ($input:expr, $res:expr) => {
        ::claim::assert_ok!(&$res);
        assert_eq!($input.eof(), $res.as_ref().unwrap().0);
    };
}

#[macro_export]
macro_rules! assert_destructure {
    {let $pat:pat = $val:expr; $asserts:block } => {
        if let $pat = $val {
            $asserts
        } else {
            panic!(
                "assertion failed, expression doesn't match pattern.\nexpected: {}\nactual: {:#?}",
                stringify!($pat),
                $val
            );
        }
    };
}

#[macro_export]
macro_rules! assert_parse_failed {
    ($input:expr, $res:expr, offset $offset:expr, at $fragment:expr) => {
        let input_slice = $input
            .span
            .fragment()
            .get($offset..$offset + $fragment.len())
            .expect(&format!(
                "Invalid range {}..{} to input {:#?}",
                $offset,
                $offset + $fragment.len(),
                $input.as_span().fragment()
            ));
        assert_eq!(
            $fragment,
            input_slice,
            "Expected input at range {}..{} to start with {:#?} but instead got {:#?}",
            $offset,
            $offset + $fragment.len(),
            $fragment,
            input_slice,
        );
        ::claim::assert_err!(&$res);
        let err = $res.unwrap_err();
        ::claim::assert_matches!(err, ::nom::Err::Error(_) | ::nom::Err::Failure(_));
        match err {
            ::nom::Err::Error(err) | ::nom::Err::Failure(err) =>
            {
                assert_eq!($input.slice($offset..$offset + $fragment.len()), err.0);
            },
            ::nom::Err::Incomplete(_) => {
                unreachable!();
            }
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_assert_destructure() {
        let res: Result<_, ()> = Ok(5);
        assert_destructure! {
            let Ok(x) = res;
            {
                assert_gt!(x, 3);
            }
        };
    }
}
