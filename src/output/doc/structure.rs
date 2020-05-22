use super::blocks::*;
use super::inlines::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Doc {
    meta: Meta,
    content: Blocks,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Meta {}

pub type Blocks = Vec<DocBlock>;

#[derive(Debug, Clone, PartialEq)]
pub struct DocBlock {
    block: Block,
    meta: BlockMeta,
}

impl From<Block> for DocBlock {
    fn from(block: Block) -> Self {
        Self {
            block,
            meta: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlockMeta {}

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
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocInline {
    inline: Inline,
    meta: InlineMeta,
}

impl From<Inline> for DocInline {
    fn from(inline: Inline) -> Self {
        Self {
            inline,
            meta: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct InlineMeta {}

pub type Inlines = Vec<DocInline>;

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
