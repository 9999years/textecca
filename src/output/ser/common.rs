use std::io::{self, Write};

use super::super::doc::Doc;

pub trait Serializer<W: Write> {
    fn write_doc(&mut self, writer: W, doc: Doc);
}
