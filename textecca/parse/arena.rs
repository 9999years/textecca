use std::ops::Deref;

use typed_arena::Arena;

use super::Span;

pub struct Source {
    src: String,
    arena: Arena<String>,
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

    #[allow(clippy::mut_from_ref)]
    pub fn alloc(&self, val: String) -> &mut str {
        self.arena.alloc(val)
    }

    pub fn alloc_span<'i>(&'i self, val: String, loc: Span<'i>) -> Span<'i> {
        let fragment = self.arena.alloc(val);
        unsafe {
            Span::new_from_raw_offset(loc.location_offset(), loc.location_line(), fragment, ())
        }
    }
}
