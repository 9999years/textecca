//! Friendlier bindings to the [`html5ever`][html5ever] crate.
//!
//! [html5ever]: https://docs.rs/html5ever/

#![deny(missing_docs)]

use std::fmt;
use std::io::{self, Write};
use std::iter;

mod h5 {
    pub use html5ever::{interface::QualName, serialize::*, LocalName};
}
use h5::Serializer as _;
use html5ever::{namespace_url, ns};

/// An HTML serializer.
pub struct HtmlSerializer<W: Write> {
    ser: h5::HtmlSerializer<W>,
    elems: Vec<h5::QualName>,
}

impl<W: Write + fmt::Debug> fmt::Debug for HtmlSerializer<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HtmlSerializer")
            .field("writer", &self.ser.writer)
            .field("elems", &self.elems)
            .finish()
    }
}

impl<W: Write> HtmlSerializer<W> {
    /// Create a new serializer from a given `Write` object.
    pub fn new(writer: W) -> Self {
        Self {
            ser: h5::HtmlSerializer::new(
                writer,
                h5::SerializeOpts {
                    // Documentation is unclear, but the only effect this has is to
                    // not escape text when we use `write_text` within a
                    // `<noscript>` element.
                    scripting_enabled: true,
                    // Don't serialize the "root node".
                    traversal_scope: h5::TraversalScope::ChildrenOnly(None),
                    // Just panic if we attempt to serialize an invalid tree --
                    // though our bindings prevent us from doing that.
                    create_missing_parent: false,
                },
            ),
            // Will *likely* not need to reallocate.
            elems: Vec::with_capacity(256),
        }
    }

    /// Create a new serializer and write `<!DOCTYPE html>` before returning it.
    pub fn with_doctype(writer: W) -> io::Result<Self> {
        let mut ret = Self::new(writer);
        ret.write_doctype("html")?;
        ret.write_text("\n")?;
        Ok(ret)
    }

    /// Serialize a comment.
    #[must_use]
    pub fn write_comment(&mut self, text: &str) -> io::Result<()> {
        self.ser.write_comment(text)
    }

    /// Serialize text, escaping it if necessary.
    #[must_use]
    pub fn write_text(&mut self, text: impl AsRef<str>) -> io::Result<()> {
        self.ser.write_text(text.as_ref())
    }

    /// Serialize a doctype.
    #[must_use]
    pub fn write_doctype(&mut self, name: &str) -> io::Result<()> {
        self.ser.write_doctype(name)
    }

    /// Serialize the start of an element.
    #[must_use]
    pub fn elem(&mut self, name: impl AsRef<str>) -> io::Result<()> {
        let tag_name = html_name(&name);
        // We'll need to close a non-void tag.
        if !is_void(&name) {
            self.elems.push(tag_name.clone());
        }
        self.ser.start_elem(tag_name, iter::empty())
    }

    /// Serialize the start of an element with attributes.
    #[must_use]
    pub fn elem_attrs(
        &mut self,
        name: impl AsRef<str>,
        attrs: &[(impl AsRef<str>, impl AsRef<str>)],
    ) -> io::Result<()> {
        // This isn't a big deal, especially if the iterator has a size hint.
        let attrs: Vec<_> = attrs
            .into_iter()
            .map(|(name, value)| (attr_name(name), value))
            .collect();
        let tag_name = html_name(&name);
        if !is_void(&name) {
            self.elems.push(tag_name.clone());
        }
        self.ser.start_elem(
            tag_name,
            attrs.iter().map(|(name, value)| (name, value.as_ref())),
        )
    }

    /// Close the last-opened element.
    #[must_use]
    pub fn end_elem(&mut self) -> io::Result<()> {
        self.ser.end_elem(self.elems.pop().unwrap())
    }
}

/// [Void elements][void] have no children or ending tag.
///
/// [void]: https://html.spec.whatwg.org/multipage/syntax.html#void-elements
const VOID_ELEMENTS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

fn is_void(name: &impl AsRef<str>) -> bool {
    VOID_ELEMENTS.contains(&name.as_ref())
}

fn attr_name(name: impl AsRef<str>) -> h5::QualName {
    h5::QualName::new(None, ns!(), h5::LocalName::from(name.as_ref()))
}

fn html_name(name: impl AsRef<str>) -> h5::QualName {
    h5::QualName::new(None, ns!(html), h5::LocalName::from(name.as_ref()))
}
