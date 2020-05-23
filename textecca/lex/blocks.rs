use crate::lex::{
    tokenize::{Token, Tokens},
    Span,
};

#[derive(Clone, Debug, PartialEq)]
pub enum BlockChild<'i> {
    Token(Token<'i>),
    Block(Block<'i>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block<'i> {
    indent: Span<'i>,
    contents: Vec<BlockChild<'i>>,
}

impl<'i> Block<'i> {
    pub fn new(indent: Span<'i>, contents: Vec<BlockChild<'i>>) -> Self {
        Block { indent, contents }
    }
}
