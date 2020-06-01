use std::io::{self, Write};
use std::iter;
use std::vec;

// html5ever renames to avoid conflicts.
mod h5 {
    pub use html5ever::{interface::QualName, serialize::*, LocalName};
}

use h5::Serializer as _;
use html5ever::{local_name, namespace_url, ns};

use tera::Tera;

use super::{InitSerializer, Serializer, SerializerError};
use crate::doc::{Block, Doc, Inline, Inlines};

mod slugify;
pub use slugify::*;

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
                Inline::Styled { .. } => todo!(),
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
                    let tag_name = match heading.level {
                        1 => html_name!("h1"),
                        2 => html_name!("h2"),
                        3 => html_name!("h3"),
                        4 => html_name!("h4"),
                        5 => html_name!("h5"),
                        6 => html_name!("h6"),
                        _ => {
                            panic!("Bad heading level!");
                        }
                    };
                    self.ser.start_elem(tag_name.clone(), iter::empty())?;

                    // Write the URL slug.
                    let slug = slugify(&heading.text);
                    let hash_slug = format!("#{}", slug);
                    let attr_href = h5::QualName::new(None, ns!(), h5::LocalName::from("href"));
                    let attr_id = h5::QualName::new(None, ns!(), h5::LocalName::from("id"));
                    let attrs = vec![(&attr_href, hash_slug.as_str()), (&attr_id, slug.as_str())];
                    self.ser.start_elem(html_name!("a"), attrs.into_iter())?;
                    self.ser.end_elem(html_name!("a"))?;

                    self.write_inlines(heading.text)?;
                    self.ser.end_elem(tag_name)?;
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
