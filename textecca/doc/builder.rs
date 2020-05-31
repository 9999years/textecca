use std::mem;

use thiserror::Error;

use super::{
    Block, Blocks, Code, Defn, Doc, Figure, Heading, Inline, Inlines, List, ListItem, Table,
    TableCell, TermListItem,
};

/// A builder for `Doc` instances; `Command`s use a `DocBuilder` to add blocks to an output stream.
pub struct DocBuilder {
    doc: Doc,
    current: Inlines,
}

impl DocBuilder {
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
                    label: None,
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

            Block::Rule => return Ok(Some(Self::to_block(current))),

            Block::Code(code) => Self::add_to_code(current, code),
            Block::List(list) => Self::add_to_list(current, list)?,
            Block::Table(table) => Self::add_to_table(current, table),
            Block::TermList(list) => Self::add_to_termlist(current, list)?,
        }
        Ok(None)
    }

    fn drain_current(&mut self) -> Result<(), DocBuilderError> {
        Self::add_to_blocks(&mut self.current, &mut self.doc.content)
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

impl DocBuilderPush<Inline> for DocBuilder {
    fn push(&mut self, elem: Inline) -> Result<(), DocBuilderError> {
        self.current.push(elem);
        Ok(())
    }
}

/// An error while building a document.
#[derive(Error, Debug)]
pub enum DocBuilderError {
    /// Attempted to push inline data to an empty TermList.
    #[error("Attempted to push inlines to empty termlist -- what would the new item's term be?")]
    EmptyTermList,
}
