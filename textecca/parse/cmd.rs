use super::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Command<'i> {
    name: Span<'i>,
    args: Vec<Argument<'i>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Argument<'i> {
    name: Option<Span<'i>>,
    value: Span<'i>,
}

impl<'i> Argument<'i> {
    pub fn new(name: Option<Span<'i>>, value: Span<'i>) -> Self {
        Argument { name, value }
    }

    pub fn from_value(value: Span<'i>) -> Self {
        Argument { name: None, value }
    }
}
