use std::mem;

use thiserror::Error;

use super::{Block, Blocks, Defn, Doc, Figure, Heading, Inline, Inlines, ListItem};

/// A builder for `Doc` instances; `Command`s use a `DocBuilder` to add blocks to an output stream.
pub struct DocBuilder {
    doc: Doc,
    current: Inlines,
}

impl DocBuilder {
    fn current_to_block(current: &mut Inlines) -> Block {
        let old_current = mem::take(current);
        Block::Par(old_current)
    }

    fn drain_current_to_blocks(current: &mut Inlines, blocks: &mut Blocks) {
        match blocks.last_mut() {
            None => {
                blocks.push(Self::current_to_block(current));
            }
            Some(block) => {
                if let Some(new_block) = Self::drain_current_to_block(current, block) {
                    blocks.push(new_block);
                }
            }
        }
    }

    fn drain_current_to_block(current: &mut Inlines, block: &mut Block) -> Option<Block> {
        match block {
            Block::Plain(inlines)
            | Block::Par(inlines)
            | Block::Heading(Heading { text: inlines, .. })
            | Block::Figure(Figure {
                caption: inlines, ..
            }) => {
                inlines.append(current);
                None
            }
            Block::Code(_) => None,
            Block::Quote(blocks)
            | Block::Tagged(blocks)
            | Block::Defn(Defn {
                content: blocks, ..
            }) => {
                Self::drain_current_to_blocks(current, blocks);
                None
            }
            Block::List(list) => {
                match list.items.last_mut() {
                    None => {
                        list.items.push(ListItem {
                            label: None,
                            content: vec![Self::current_to_block(current)],
                        });
                    }
                    Some(item) => {
                        Self::drain_current_to_blocks(current, &mut item.content);
                    }
                }
                None
            }
            Block::Rule => Some(Self::current_to_block(current)),
            Block::Table(table) => None,
            Block::TermList(_) => None,
        }
    }

    fn drain_current(&mut self) {
        Self::drain_current_to_blocks(&mut self.current, &mut self.doc.content);
    }
}

/// Helper trait for `DocBuilder` to encapsulate pushing either `Block` or `Inline` values.
pub trait DocBuilderPush<T> {
    /// Add the given element, either a `Block` or an `Inline`, to the document.
    fn push(&mut self, elem: T) -> Result<(), DocBuilderError>;
}

impl DocBuilderPush<Block> for DocBuilder {
    fn push(&mut self, elem: Block) -> Result<(), DocBuilderError> {
        Ok(())
    }
}

impl DocBuilderPush<Inline> for DocBuilder {
    fn push(&mut self, elem: Inline) -> Result<(), DocBuilderError> {
        Ok(())
    }
}

/// An error while building a document.
#[derive(Error, Debug)]
pub enum DocBuilderError {}
