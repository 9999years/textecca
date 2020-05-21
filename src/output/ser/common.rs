use std::io::{self, Write};

use super::super::doc::Doc;

pub trait Serializer {
    fn write_doc<W: Write>(&mut self, writer: W, doc: Doc);
}
