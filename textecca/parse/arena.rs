use std::error::Error;
use std::ops::Deref;

use derivative::Derivative;
use typed_arena::Arena;

use super::{Parser, Span, Tokens};

/// Source code tied to an arena allocator of strings.
///
/// Because textecca `Parser`s may produce tokens unrelated to (or at least
/// containing text not found in) their input, they need to be able to produce
/// `Span`s of arbitrary text with the same lifetime as the source. By bundling
/// an arena allocator with the owned source code, an `&'i Source` reference will
/// allow extracting `Span<'i>`s from the input or creating novel `Span<'i>`s
/// with the allocator.
///
/// No, a `Cow<'i, str>` doesn't work here, unfortunately.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Source {
    src: String,
    #[derivative(Debug = "ignore")]
    arena: Arena<String>,
}

impl Clone for Source {
    fn clone(&self) -> Self {
        Source::new(self.src.clone())
    }
}

impl PartialEq for Source {
    fn eq(&self, other: &Self) -> bool {
        self.src.eq(&other.src)
    }
}

impl From<String> for Source {
    fn from(s: String) -> Self {
        Source::new(s)
    }
}

impl<'i> Into<Span<'i>> for &'i Source {
    fn into(self) -> Span<'i> {
        Span::new(&self.src)
    }
}

impl Deref for Source {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.src
    }
}

impl Source {
    /// Create a new source-arena.
    pub fn new(src: String) -> Self {
        let cap = src.len() / 16;
        Self::with_capacity(src, cap)
    }

    /// Create a new source-arena with the given capacity for new tokens.
    pub fn with_capacity(src: String, n: usize) -> Self {
        Self {
            src,
            arena: Arena::with_capacity(n),
        }
    }

    /// Allocate a string and return a mutable reference to it.
    ///
    /// This is useful for creating new tokens with the same lifespan as the input.
    #[allow(clippy::mut_from_ref)]
    pub fn alloc(&self, val: String) -> &mut str {
        self.arena.alloc(val)
    }

    /// Allocate a span with the given text, using an existing span for the
    /// location.
    pub fn alloc_span<'i>(&'i self, val: String, loc: Span<'i>) -> Span<'i> {
        let fragment = self.arena.alloc(val);
        unsafe {
            Span::new_from_raw_offset(loc.location_offset(), loc.location_line(), fragment, ())
        }
    }

    /// Create a span-generation function. This helps avoid passing the arena itself around.
    pub fn alloc_spans<'i>(&'i self, val: String) -> impl Fn(Span<'i>) -> Span<'i> + 'i {
        let fragment: &'i str = self.arena.alloc(val);
        move |loc| unsafe {
            Span::new_from_raw_offset(loc.location_offset(), loc.location_line(), fragment, ())
        }
    }
}

/// A `Parser` bundled with a `Source`-arena.
pub struct ParserArena<'i> {
    arena: &'i Source,
    parser: Parser,
}

impl<'i> ParserArena<'i> {
    /// Create a new `ParserArena` from the given `Source` and `Parser`.
    pub fn new(arena: &'i Source, parser: Parser) -> Self {
        Self { arena, parser }
    }

    /// Parse the given input with this arena's `Parser`.
    pub fn parse(&self, input: Span<'i>) -> Result<Tokens<'i>, Box<dyn Error + 'i>> {
        (self.parser)(self.arena, input)
    }
}
