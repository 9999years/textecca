use crate::lex::{Command, Span};
use crate::output::doc::{Block, Inline};

pub enum EvalBlock<'i> {
    Block(Block),
    Unparsed(Span<'i>),
}

pub enum EvalInline<'i> {
    Inline(Inline),
    Unparsed(Command<'i>),
}
