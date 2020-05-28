use std::collections::HashMap;

use super::blocks::*;
use super::inlines::*;

/// Some metadata to be associated with a group of blocks or inlines; metadata is
/// currently unstructured and its representation will almost certainly change in
/// the future.
pub type Meta = HashMap<String, String>;

/// An entire document.
#[derive(Debug, Clone, PartialEq)]
pub struct Doc {
    pub meta: DocMeta,
    pub content: Blocks,
}

/// Document metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct DocMeta {}

/// A sequence of `Block`s.
pub type Blocks = Vec<Block>;

/// A block of content within a document, typically separated by vertical space.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// Text not in a paragraph; this is treated as `Inlines`, but in a block context.
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

    /// Blocks tagged with some metadata.
    Tagged(Blocks),
}

/// A sequence of `Inline`s.
pub type Inlines = Vec<Inline>;

/// A span of inline content in a document.
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    /// Plain text.
    Text(String),

    /// Style instruction.
    Styled(Style),

    /// An inline quotation.
    Quote(Quote),

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
