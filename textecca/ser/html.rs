use std::io::{self, Write};
use std::iter;

// html5ever renames to avoid conflicts.
mod h5 {
    pub use html5ever::{interface::QualName, serialize::*};
}

use h5::Serializer as _;
use html5ever::{local_name, namespace_url, ns};

use tera::Tera;

use super::{InitSerializer, Serializer, SerializerError};
use crate::doc::{Block, Doc, Inline, Inlines};

#[allow(unused_macros)]
macro_rules! html_name {
    ($el_name:tt) => {
        h5::QualName::new(None, ns!(html), local_name!($el_name))
    };
}

/// Serializer to HTML5.
pub struct HtmlSerializer<W: Write> {
    ser: h5::HtmlSerializer<W>,
    seen_paragraph: bool,
    tera: Tera,
}

impl<W: Write> InitSerializer<W> for HtmlSerializer<W> {
    fn new(writer: W) -> Result<Box<Self>, SerializerError> {
        Ok(Box::new(Self {
            seen_paragraph: false,
            ser: h5::HtmlSerializer::new(
                writer,
                h5::SerializeOpts {
                    create_missing_parent: false,
                    scripting_enabled: false,
                    traversal_scope: h5::TraversalScope::ChildrenOnly(None),
                },
            ),
            // TODO: Don't hardcode templates directory.
            tera: Tera::new("templates/*").map_err(|e| SerializerError::Other(Box::new(e)))?,
        }))
    }
}

impl<W: Write> HtmlSerializer<W> {
    fn write_inlines(&mut self, inlines: Inlines) -> Result<(), SerializerError> {
        for inline in inlines {
            match inline {
                Inline::Text(text) => {
                    self.ser.write_text(&text)?;
                }
                Inline::Styled(_) => todo!(),
                Inline::Quote(_) => todo!(),
                Inline::Code(_) => todo!(),
                Inline::Space => {
                    self.ser.write_text(" ")?;
                }
                Inline::Link(_) => todo!(),
                Inline::Footnote(_) => todo!(),
                Inline::Math(_) => todo!(),
            }
        }
        Ok(())
    }
}

impl<W: Write> Serializer for HtmlSerializer<W> {
    fn write_doc(&mut self, doc: Doc) -> Result<(), SerializerError> {
        self.ser.write_doctype("html")?;
        self.ser.write_text("\n")?;

        let mut base_ctx = tera::Context::new();
        for (k, v) in doc.meta {
            base_ctx.insert(k, &v);
        }

        for block in doc.content {
            match block {
                Block::Plain(inlines) => {
                    self.write_inlines(inlines)?;
                }
                Block::Par(inlines) => {
                    if self.seen_paragraph {
                        self.ser.end_elem(html_name!("p"))?;
                    }
                    self.ser.write_text("\n")?;
                    self.ser.start_elem(html_name!("p"), iter::empty())?;
                    self.write_inlines(inlines)?;
                }
                Block::Code(_) => todo!(),
                Block::Quote(_) => todo!(),
                Block::List(_) => todo!(),
                Block::Heading(heading) => {
                    let mut ctx = base_ctx.clone();
                    ctx.insert("level", &heading.level);
                    // ctx.insert("")
                }
                Block::Rule => {
                    self.ser.start_elem(html_name!("hr"), iter::empty())?;
                }
                Block::Table(_) => todo!(),
                Block::Figure(_) => todo!(),
                Block::Defn(_) => todo!(),
                Block::Tagged(_) => todo!(),
                Block::TermList(_) => todo!(),
            }
        }

        Ok(())
    }
}
