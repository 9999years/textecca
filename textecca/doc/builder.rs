use super::{Block, Doc};

/// A builder for `Doc` instances; `Command`s use a `DocBuilder` to add blocks to an output stream.
pub struct DocBuilder {
    doc: Doc,
}

impl DocBuilder {
    /// Add the given `Block` to the document.
    pub fn push(&mut self, block: Block) {
        self.doc.content.push(block)
    }
}
