use super::Span;

/// A parsed command, consisting of a name and arguments.
#[derive(Clone, Debug, PartialEq)]
pub struct Command<'i> {
    name: Span<'i>,
    args: Vec<Argument<'i>>,
}

/// An argument to a command.
#[derive(Clone, Debug, PartialEq)]
pub struct Argument<'i> {
    name: Option<Span<'i>>,
    value: Span<'i>,
}

impl<'i> Argument<'i> {
    /// Create a new `Argument`.
    pub fn new(name: Option<Span<'i>>, value: Span<'i>) -> Self {
        Argument { name, value }
    }

    /// Create a new `Argument` with no explicit name.
    pub fn from_value(value: Span<'i>) -> Self {
        Argument { name: None, value }
    }
}
