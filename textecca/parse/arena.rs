use std::error::Error;
use std::ops::Deref;

use derivative::Derivative;
use typed_arena::Arena;

use super::{Parser, RawTokens, Span, Tokens};

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
    pub fn new(src: String) -> Self {
        let cap = src.len() / 16;
        Self::with_capacity(src, cap)
    }

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

pub struct ParserArena<'i> {
    arena: &'i Source,
    parser: Parser,
}

impl<'i> ParserArena<'i> {
    pub fn new(arena: &'i Source, parser: Parser) -> Self {
        Self { arena, parser }
    }

    pub fn parse(&self, input: Span<'i>) -> Result<Tokens<'i>, Box<dyn Error + 'i>> {
        (self.parser)(self.arena, input)
    }
}
