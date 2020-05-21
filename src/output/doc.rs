use super::length::Length;

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
    meta: (),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockMeta {}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// Paragraph.
    Para(String),

    /// Code block.
    Code(String),

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
pub struct Table {
    columns: Vec<TableColumn>,
    cells: Vec<Vec<TableCell>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    alignment: Option<Alignment>,
    row_span: i64,
    col_span: i64,
    content: Blocks,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    alignment: Alignment,
    /// Relative width.
    width: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Alignment {}

#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    level: i32,
    text: Inlines,
}

#[derive(Debug, Clone, PartialEq)]
pub struct List {
    kind: ListKind,
    items: Vec<ListItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListKind {
    Unordered,
    Ordered,
    Definition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    label: Option<Inlines>,
    content: Blocks,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocInline {
    inline: Inline,
    meta: InlineMeta,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    text: Inlines,
    label: Option<String>,
    target: String,
}

/// Maybe at some point in the future.
#[derive(Debug, Clone, PartialEq)]
pub struct Citation {}

#[derive(Debug, Clone, PartialEq)]
pub struct Quote {
    kind: QuoteKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuoteKind {
    Single,
    Double,
    /// Left, right quote markers.
    Other(Box<DocInline>, Box<DocInline>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Style {
    Emph,
    Strong,
    Superscript,
    Subscript,
    SmallCaps,
    Strikeout,
    Underline,
    Size(Length),
    Color(Color),
    Font(Font),
    FontFeatures(FontFeatures),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Color {}
#[derive(Debug, Clone, PartialEq)]
pub struct Font {}
#[derive(Debug, Clone, PartialEq)]
pub struct FontFeatures {}
