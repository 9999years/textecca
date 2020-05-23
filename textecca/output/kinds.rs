use std::io;

use html5ever::serialize::Serializer;

pub enum OutputKind {
    HTML,
}

trait HTMLOutput {
    fn output<S: Serializer>(&self, doc: S) -> io::Result<()>;
}
