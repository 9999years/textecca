//! Serialization of documents to various formats.
use std::io::{self, Write};

use thiserror::Error;

use crate::doc::Block;
use crate::doc::Doc;

/// An error while serializing a document.
#[derive(Error, Debug)]
pub enum SerializerError {
    #[error("{0}")]
    Io(io::Error),

    #[error("Unsupported block {0:?}")]
    Unsupported(Block),
}

/// A document serializer for a particular format.
pub trait Serializer {
    fn write_doc<W: Write>(&mut self, writer: W, doc: Doc) -> Result<(), SerializerError>;
}
