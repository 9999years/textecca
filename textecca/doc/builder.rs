use std::{convert::TryInto, mem};

use thiserror::Error;

use super::{
    Block, Blocks, Code, Defn, Doc, Figure, Heading, Inline, Inlines, List, ListItem, Table,
    TableCell, TermListItem,
};
use crate::parse::Span;

/// A builder for `Doc` instances; `Command`s use a `DocBuilder` to add blocks to an output stream.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DocBuilder {
    doc: Doc,
    current: Inlines,
}

impl TryInto<Doc> for DocBuilder {
    type Error = DocBuilderError;
    fn try_into(self) -> Result<Doc, Self::Error> {
        let mut self_ = self;
        self_.drain_current()?;
        Ok(self_.doc)
    }
}

impl TryInto<Blocks> for DocBuilder {
    type Error = DocBuilderError;
    fn try_into(self) -> Result<Blocks, Self::Error> {
        let doc: Doc = self.try_into()?;
        Ok(doc.content)
    }
}

impl TryInto<Inlines> for DocBuilder {
    type Error = DocBuilderError;
    fn try_into(self) -> Result<Inlines, Self::Error> {
        let doc: Doc = self.try_into()?;
        let mut blocks = doc.content;

        if blocks.is_empty() {
            Ok(Vec::new())
        } else {
            let block = blocks
                .pop()
                .ok_or_else(|| DocBuilderError::UnexpectedBlocks(blocks))?;
            match block {
                Block::Plain(inlines) | Block::Par(inlines) => Ok(inlines),
                _ => Err(DocBuilderError::UnexpectedBlocks(vec![block])),
            }
        }
    }
}

impl DocBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a new builder inheriting from the given parent.
    pub fn new_inheriting(_parent: &Self) -> Self {
        Default::default()
    }

    fn to_block(current: &mut Inlines) -> Block {
        Block::Par(mem::take(current))
    }

    fn add_to_blocks(current: &mut Inlines, blocks: &mut Blocks) -> Result<(), DocBuilderError> {
        match blocks.last_mut() {
            None => {
                blocks.push(Self::to_block(current));
            }
            Some(block) => {
                if let Some(new_block) = Self::add_to_block(current, block)? {
                    blocks.push(new_block);
                }
            }
        }
        Ok(())
    }

    fn add_to_list(current: &mut Inlines, list: &mut List) -> Result<(), DocBuilderError> {
        match list.items.last_mut() {
            None => {
                list.items.push(ListItem {
                    content: vec![Self::to_block(current)],
                });
                Ok(())
            }
            Some(item) => Self::add_to_blocks(current, &mut item.content),
        }
    }

    fn add_to_termlist(
        current: &mut Inlines,
        list: &mut Vec<TermListItem>,
    ) -> Result<(), DocBuilderError> {
        match list.last_mut() {
            None => Err(DocBuilderError::EmptyTermList),
            Some(item) => {
                Self::add_to_blocks(current, &mut item.content)?;
                Ok(())
            }
        }
    }

    fn add_to_table(current: &mut Inlines, table: &mut Table) {
        match table.cells.last_mut().and_then(|row| row.last_mut()) {
            None => {
                let mut row = Vec::with_capacity(table.columns.len());
                row.push(TableCell {
                    content: vec![Block::Plain(mem::take(current))],
                    ..Default::default()
                });
                table.cells.push(row);
            }
            Some(_) => {}
        }
    }

    fn add_to_code(current: &mut Inlines, code: &mut Code) {
        match code.lines.last_mut() {
            None => {
                code.lines.push(mem::take(current));
            }
            Some(inlines) => {
                inlines.append(current);
            }
        }
    }

    fn add_to_block(
        current: &mut Inlines,
        block: &mut Block,
    ) -> Result<Option<Block>, DocBuilderError> {
        match block {
            Block::Plain(inlines)
            | Block::Par(inlines)
            | Block::Heading(Heading { text: inlines, .. })
            | Block::Figure(Figure {
                caption: inlines, ..
            }) => {
                inlines.append(current);
            }

            Block::Quote(blocks)
            | Block::Tagged(blocks)
            | Block::Defn(Defn {
                content: blocks, ..
            }) => {
                Self::add_to_blocks(current, blocks)?;
            }

            Block::Rule | Block::Math(_) => return Ok(Some(Self::to_block(current))),

            Block::Code(code) => Self::add_to_code(current, code),
            Block::List(list) => Self::add_to_list(current, list)?,
            Block::Table(table) => Self::add_to_table(current, table),
            Block::TermList(list) => Self::add_to_termlist(current, list)?,
        }
        Ok(None)
    }

    fn drain_current(&mut self) -> Result<(), DocBuilderError> {
        if !self.current.is_empty() {
            Self::add_to_blocks(&mut self.current, &mut self.doc.content)
        } else {
            Ok(())
        }
    }
}

/// Helper trait for `DocBuilder` to encapsulate pushing either `Block` or `Inline` values.
pub trait DocBuilderPush<T> {
    /// Add the given element, either a `Block` or an `Inline`, to the document.
    fn push(&mut self, elem: T) -> Result<(), DocBuilderError>;
}

impl DocBuilderPush<Block> for DocBuilder {
    fn push(&mut self, elem: Block) -> Result<(), DocBuilderError> {
        self.drain_current()?;
        self.doc.content.push(elem);
        Ok(())
    }
}

impl DocBuilderPush<Blocks> for DocBuilder {
    fn push(&mut self, elem: Blocks) -> Result<(), DocBuilderError> {
        self.drain_current()?;
        let mut elem = elem;
        self.doc.content.append(&mut elem);
        Ok(())
    }
}

impl DocBuilderPush<Inline> for DocBuilder {
    fn push(&mut self, elem: Inline) -> Result<(), DocBuilderError> {
        self.current.push(elem);
        Ok(())
    }
}

impl DocBuilderPush<Inlines> for DocBuilder {
    fn push(&mut self, elem: Inlines) -> Result<(), DocBuilderError> {
        let mut elem = elem;
        self.current.append(&mut elem);
        Ok(())
    }
}

impl<'i> DocBuilderPush<Span<'i>> for DocBuilder {
    fn push(&mut self, elem: Span<'i>) -> Result<(), DocBuilderError> {
        self.current.push(Inline::Text(elem.fragment().to_string()));
        Ok(())
    }
}

/// An error while building a document.
#[derive(Error, Debug)]
pub enum DocBuilderError {
    /// Attempted to push inline data to an empty TermList.
    #[error("Attempted to push inlines to empty termlist")]
    EmptyTermList,

    /// Inlines were expected.
    #[error("Unexpected blocks {0:?}")]
    UnexpectedBlocks(Blocks),
}
