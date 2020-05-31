//! Serialization of documents to various formats.
use std::error;
use std::io::{self, Write};

use thiserror::Error;

use crate::doc::Block;
use crate::doc::Doc;

mod html;
pub use html::*;

/// An error while serializing a document.
#[derive(Error, Debug)]
pub enum SerializerError {
    /// An IO error.
    #[error("{0}")]
    Io(#[from] io::Error),

    /// Some other arbitrary error.
    #[error("{0}")]
    Other(Box<dyn error::Error>),
}

/// Trait to initialize a `Serializer`.
pub trait InitSerializer<W: Write> {
    /// Create a new `Serializer` from the given basename.
    fn new(writer: W) -> Result<Box<Self>, SerializerError>;
}

/// A document serializer for a particular format.
pub trait Serializer {
    /// Serialize the given document.
    fn write_doc(&mut self, doc: Doc) -> Result<(), SerializerError>;
}
