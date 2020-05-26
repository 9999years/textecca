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
pub enum Alignment {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    pub level: i32,
    pub text: Inlines,
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
