use std::{convert::TryInto, mem};

use thiserror::Error;

use super::{
    Block, BlockInner, Blocks, Code, Defn, Doc, Figure, Heading, Id, Inline, Inlines, List,
    ListItem, Table, TableCell, TermListItem,
};
use crate::parse::Span;

/// A builder for `Doc` instances; `Command`s use a `DocBuilder` to add blocks to an output stream.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DocBuilder {
    doc: Doc,
    inner: DocBuilderInner,
}
#[derive(Debug, Default, Clone, PartialEq)]
struct DocBuilderInner {
    current: Inlines,
    id: Id,
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
            Ok(Default::default())
        } else {
            let block = blocks
                .pop()
                .ok_or_else(|| DocBuilderError::UnexpectedBlocks(blocks))?;
            match block.inner {
                BlockInner::Plain(inlines) | BlockInner::Par(inlines) => Ok(inlines),
                _ => Err(DocBuilderError::UnexpectedBlocks(block.into())),
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

    fn drain_current(&mut self) -> Result<(), DocBuilderError> {
        if self.inner.is_empty() {
            Ok(())
        } else {
            self.inner.add_to_blocks(&mut self.doc.content)
        }
    }
}

impl DocBuilderInner {
    fn is_empty(&self) -> bool {
        self.current.is_empty()
    }

    fn inc_id(&mut self) -> Id {
        let id = self.id;
        self.id = self.id.next().unwrap();
        id
    }

    fn take_current(&mut self) -> Inlines {
        mem::take(&mut self.current)
    }

    fn block_from_inner(&mut self, inner: BlockInner) -> Block {
        Block {
            id: self.inc_id(),
            inner,
        }
    }

    fn to_block(&mut self) -> Block {
        let inner = BlockInner::Par(self.take_current());
        self.block_from_inner(inner)
    }

    fn add_to_list(&mut self, list: &mut List) -> Result<(), DocBuilderError> {
        match list.items.last_mut() {
            None => {
                list.items.push(ListItem {
                    content: self.to_block().into(),
                });
                Ok(())
            }
            Some(item) => self.add_to_blocks(&mut item.content),
        }
    }

    fn add_to_termlist(&mut self, list: &mut Vec<TermListItem>) -> Result<(), DocBuilderError> {
        match list.last_mut() {
            None => Err(DocBuilderError::EmptyTermList),
            Some(item) => {
                self.add_to_blocks(&mut item.content)?;
                Ok(())
            }
        }
    }

    fn add_to_table(&mut self, table: &mut Table) {
        match table.cells.last_mut().and_then(|row| row.last_mut()) {
            None => {
                let mut row = Vec::with_capacity(table.columns.len());
                let inner = BlockInner::Plain(self.take_current());
                row.push(TableCell {
                    content: self.block_from_inner(inner).into(),
                    ..Default::default()
                });
                table.cells.push(row);
            }
            Some(_) => {}
        }
    }

    fn add_to_code(&mut self, code: &mut Code) {
        match code.lines.last_mut() {
            None => {
                code.lines.push(self.take_current());
            }
            Some(inlines) => {
                inlines.append(&mut self.current);
            }
        }
    }

    #[must_use]
    fn add_to_block(&mut self, block: &mut BlockInner) -> Result<Option<Block>, DocBuilderError> {
        match block {
            BlockInner::Plain(inlines)
            | BlockInner::Par(inlines)
            | BlockInner::Heading(Heading { text: inlines, .. })
            | BlockInner::Figure(Figure {
                caption: inlines, ..
            }) => {
                inlines.append(&mut self.current);
            }

            BlockInner::Quote(blocks)
            | BlockInner::Defn(Defn {
                content: blocks, ..
            }) => {
                self.add_to_blocks(blocks)?;
            }

            BlockInner::Rule | BlockInner::Math(_) => return Ok(Some(self.to_block())),

            BlockInner::Code(code) => self.add_to_code(code),
            BlockInner::List(list) => self.add_to_list(list)?,
            BlockInner::Table(table) => self.add_to_table(table),
            BlockInner::TermList(list) => self.add_to_termlist(list)?,
        }
        Ok(None)
    }

    fn add_to_blocks(&mut self, blocks: &mut Blocks) -> Result<(), DocBuilderError> {
        match blocks.last_mut() {
            None => {
                blocks.push(self.to_block());
            }
            Some(block) => {
                if let Some(new_block) = self.add_to_block(block)? {
                    blocks.push(new_block);
                }
            }
        }
        Ok(())
    }
}

/// Helper trait for `DocBuilder` to encapsulate pushing either `Block` or `Inline` values.
pub trait DocBuilderPush<T> {
    /// Add the given element, either a `Block` or an `Inline`, to the document.
    fn push(&mut self, elem: T) -> Result<(), DocBuilderError>;
}

impl DocBuilderPush<BlockInner> for DocBuilder {
    fn push(&mut self, elem: BlockInner) -> Result<(), DocBuilderError> {
        self.drain_current()?;
        self.doc.content.push(self.inner.block_from_inner(elem));
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
        self.inner.current.push(elem);
        Ok(())
    }
}

impl DocBuilderPush<Inlines> for DocBuilder {
    fn push(&mut self, elem: Inlines) -> Result<(), DocBuilderError> {
        let mut elem = elem;
        self.inner.current.append(&mut elem);
        Ok(())
    }
}

impl<'i> DocBuilderPush<Span<'i>> for DocBuilder {
    fn push(&mut self, elem: Span<'i>) -> Result<(), DocBuilderError> {
        self.inner
            .current
            .push(Inline::Text(elem.fragment().to_string()));
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
