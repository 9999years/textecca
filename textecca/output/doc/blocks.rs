use super::structure::{Blocks, Inlines};

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
