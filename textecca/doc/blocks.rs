use std::collections::HashMap;

use super::structure::{Blocks, Inlines, Meta};

/// A group of blocks tagged with some metadata; metadata is currently
/// unstructured and its representation will almost certainly change in the
/// future.
#[derive(Debug, Clone, PartialEq)]
pub struct TaggedBlocks {
    content: Blocks,
    meta: Meta,
}

/// A table.
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    /// The table's column specifications; this field holds no data, but
    /// determines the table's display.
    pub columns: Vec<TableColumn>,

    /// The table's data. Note that due to row- and column-spans, some rows may not have `columns.len()` items.
    pub cells: TableRows,
}

/// A list of table rows, from top to bottom.
/// Each element in a `TableRows` lists a row's cells from left to right.
pub type TableRows = Vec<Vec<TableCell>>;

/// A cell in a `Table`.
#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    /// The cell's alignment.
    pub alignment: Option<Alignment>,
    /// The cell's row-span, the number of rows (vertically) the cell encompasses.
    ///
    /// A row-span of 1 indicates this is a normal cell. Row-spans should not be 0.
    pub row_span: u32,
    /// The cell's column-span.
    pub col_span: u32,
    /// The cell's contents.
    pub content: Blocks,
}

impl Default for TableCell {
    fn default() -> Self {
        Self {
            alignment: None,
            row_span: 1,
            col_span: 1,
            content: Vec::new(),
        }
    }
}

/// A column-specification in a `Table`; note that this does *not* include the
/// column's *contents.*
#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    /// The column's alignment.
    pub alignment: Alignment,
    /// Relative width.
    pub width: f64,
}

/// A `Table` column's alignment.
#[derive(Debug, Clone, PartialEq)]
pub enum Alignment {
    /// Left-aligned.
    Left,
    /// Right-aligned.
    Right,
    /// Center-aligned.
    Center,
    /// Justified.
    Justify,
}

/// A document heading.
#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    /// The heading's level in the document hierarchy.
    pub level: i32,
    /// The heading's text.
    pub text: Inlines,
}

#[derive(Debug, Clone, PartialEq)]
enum HeadingLevel {
    /// The main title of a document; only one should exist per document.
    MainTitle = -3,
    /// A part in a document, consisting of multiple chapters. Generally, a
    /// part-heading is a full page.
    Part = -2,
    /// A chapter in a document.
    Chapter = -1,
    /// A section in a document; most smaller documents will have only `Section`
    /// and below headings.
    Section = 1,
    /// A subsection in a document.
    Subsection = 2,
    /// A subsubsection in a document.
    Subsubsection = 3,
    /// A paragraph-level heading. These headings are rare and should be
    /// used sparingly if ever.
    Paragraph = 4,
    /// A sentence-level heading. Why?
    Sentence = 5,
}

/// A list, ordered, unordered, or of defined terms.
#[derive(Debug, Clone, PartialEq)]
pub struct List {
    /// The list's kind.
    pub kind: ListKind,
    /// The list's items.
    pub items: Vec<ListItem>,
}

/// A `List`'s type.
#[derive(Debug, Clone, PartialEq)]
pub enum ListKind {
    /// An unordered, i.e. bulleted list.
    ///
    /// In HTML, this corresponds to the [`ul`][ul] element, and in LaTeX this
    /// corresponds to the `itemize` environment.
    ///
    /// [ul]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ul
    Unordered,
    /// An ordered, i.e. numbered list.
    ///
    /// In HTML, this corresponds to the [`ol`][ol] element, and in LaTeX this
    /// corresponds to the `enumerate` environment.
    ///
    /// [ol]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ol
    Ordered,
    /// A description/definition list, used for defining specific terms.
    ///
    /// In HTML, this corresponds to the [`<dl>`][dl] element, and in LaTeX this
    /// corresponds to the `description` environment.
    ///
    /// [dl]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dl
    Description,
}

/// An item in a `List`.
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    /// This item's label; if empty, the `Serializer` may substitute any value it
    /// sees fit.
    pub label: Option<Inlines>,
    /// This item's content.
    pub content: Blocks,
}

/// A list, ordered, unordered, or of defined terms.
#[derive(Debug, Clone, PartialEq)]
pub struct TermListItem {
    /// This item's label.
    pub term: Inlines,
    /// This item's content.
    pub content: Blocks,
}

/// A figure, i.e. a captioned diagram, image, or similar.
#[derive(Debug, Clone, PartialEq)]
pub struct Figure {
    /// The kind of figure.
    pub kind: FigureKind,
    /// The figure's caption.
    pub caption: Inlines,
    /// The figure's content, i.e. the image/diagram/table/etc.
    pub content: Blocks,
}

/// The kind of figure, used for labelling.
#[derive(Debug, Clone, PartialEq)]
pub enum FigureKind {
    /// A figure, diagram, etc.
    Figure,
    /// A table of data.
    Table,
    /// A code listing.
    Listing,
    /// Some other value.
    Other(String),
}

/// A defined object; a definition of a term, a theorem, an article, etc.
#[derive(Debug, Clone, PartialEq)]
pub struct Defn {
    /// The defined object's name. For a term definition, this would be the term.
    /// For a Wikipedia article, it would be the article title.
    pub name: Inlines,
    /// The definition's summary. Roughly equivalent to the first sentence of a
    /// Wikipedia article. The summary allows multiple blocks for displayed
    /// equations, code snippets, or figures.
    pub summary: Blocks,
    /// Extra content; can provide more elaborate detail, examples, or other information. May be empty.
    pub content: Blocks,
}

/// A code listing.
#[derive(Debug, Clone, PartialEq)]
pub struct Code {
    /// The code's language, for highlighting. `"plain"` indicates no highlighting.
    pub language: String,
    /// The line numbering scheme to use, if any.
    pub line_numbers: Option<LineNumbers>,
    /// The lines of code themselves.
    pub lines: Vec<Inlines>,
}

/// A `Code` listing's line numbers, if any.
#[derive(Debug, Clone, PartialEq)]
pub struct LineNumbers {
    /// The starting line number.
    pub start: i32,
}
