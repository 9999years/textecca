use std::collections::HashMap;

use super::blocks::*;
use super::inlines::*;

/// Some metadata to be associated with a group of blocks or inlines; metadata is
/// currently unstructured and its representation will almost certainly change in
/// the future.
pub type Meta = HashMap<String, String>;

#[derive(Debug, Clone, PartialEq)]
pub struct Doc {
    pub meta: DocMeta,
    pub content: Blocks,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocMeta {}

pub type Blocks = Vec<Block>;

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// Text not in a paragraph.
    Plain(Inlines),

    /// Paragraph.
    Para(Inlines),

    /// Code block.
    Code(Inlines),

    /// Block quote.
    Quote(Blocks),

    /// A list; ordered, unordered, or definitions.
    List(List),

    /// A heading, or more accurately a document division.
    Heading(Heading),

    /// Horizontal rule.
    Rule,

    /// A table.
    Table(Table),

    /// A figure-like block; a diagram, image, or similar.
    Figure(Blocks),

    /// A concept; this could be a warning, definition, note, theorem, etc.
    Concept(Blocks),

    Tagged(Blocks),
}

pub type Inlines = Vec<Inline>;

#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    /// Plain text.
    Text(String),

    /// Style instruction.
    Styled(Style),

    Quote(Quote),

    /// Unsupported for now.
    Citation(Citation),

    /// Inline code span.
    Code(String),

    /// Inter-word space.
    ///
    /// TODO: How to handle inter-sentence spacing?
    Space,

    /// A link, either intra-document or external.
    Link(Link),

    /// A footnote.
    ///
    /// TODO: Endnotes, footnote positioning, end-of-chapter notes...?
    Footnote(String),

    /// Mathematics.
    Math(String),
}
