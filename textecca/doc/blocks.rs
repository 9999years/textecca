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
    columns: Vec<TableColumn>,
    cells: Vec<Vec<TableCell>>,
}

/// A cell in a `Table`.
#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    alignment: Option<Alignment>,
    row_span: i64,
    col_span: i64,
    content: Blocks,
}

/// A column-specification in a `Table`; note that this does *not* include the
/// column's *contents.*
#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    alignment: Alignment,
    /// Relative width.
    width: f64,
}

/// A `Table` column's alignment.
#[derive(Debug, Clone, PartialEq)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

/// A document heading.
#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    pub level: i32,
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
    kind: ListKind,
    items: Vec<ListItem>,
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
    label: Option<Inlines>,
    content: Blocks,
}
