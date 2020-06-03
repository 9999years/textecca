//! Friendlier bindings to the [`html5ever`][html5ever] crate.
//!
//! [html5ever]: https://docs.rs/html5ever/

#![deny(missing_docs)]

use std::fmt;
use std::io::{self, Write};
use std::{
    borrow::{Borrow, Cow},
    iter,
};

use thiserror::Error;

mod h5 {
    pub use html5ever::{interface::QualName, serialize::*, tokenizer::*, LocalName};
}
use h5::Serializer as _;
use html5ever::{namespace_url, ns};

mod tendril_ext;
use tendril_ext::AsStrLossy;

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
    pub fn with_doctype(writer: W) -> Result<Self, SerializeError> {
        let mut ret = Self::new(writer);
        ret.write_doctype("html")?;
        ret.write_text("\n")?;
        Ok(ret)
    }

    /// Serialize a comment.
    #[must_use]
    pub fn write_comment(&mut self, text: &str) -> Result<(), SerializeError> {
        Ok(self.ser.write_comment(text)?)
    }

    /// Serialize text, escaping it if necessary.
    #[must_use]
    pub fn write_text(&mut self, text: impl AsRef<str>) -> Result<(), SerializeError> {
        Ok(self.ser.write_text(text.as_ref())?)
    }

    /// Serialize a doctype.
    #[must_use]
    pub fn write_doctype(&mut self, name: &str) -> Result<(), SerializeError> {
        Ok(self.ser.write_doctype(name)?)
    }

    /// Serialize the start of an element.
    #[must_use]
    pub fn elem(&mut self, name: impl AsRef<str>) -> Result<(), SerializeError> {
        let tag_name = html_name(&name);
        // We'll need to close a non-void tag.
        let elem_is_void = is_void(&name);
        let ret = Ok(self.ser.start_elem(tag_name.clone(), iter::empty())?);
        if elem_is_void {
            self.ser.end_elem(tag_name)?;
        } else {
            self.elems.push(tag_name);
        }
        ret
    }

    /// Serialize the start of an element with attributes.
    #[must_use]
    pub fn elem_attrs(
        &mut self,
        name: impl AsRef<str>,
        attrs: &[(impl AsRef<str>, impl AsRef<str>)],
    ) -> Result<(), SerializeError> {
        // This isn't a big deal, especially if the iterator has a size hint.
        let attrs: Vec<_> = attrs
            .into_iter()
            .map(|(name, value)| (attr_name(name), value))
            .collect();
        let tag_name = html_name(&name);
        let elem_is_void = is_void(&name);
        let ret = Ok(self.ser.start_elem(
            tag_name.clone(),
            attrs.iter().map(|(name, value)| (name, value.as_ref())),
        )?);
        if elem_is_void {
            self.ser.end_elem(tag_name)?;
        } else {
            self.elems.push(tag_name);
        }
        ret
    }

    /// Close the last-opened element.
    #[must_use]
    pub fn end_elem(&mut self) -> Result<(), SerializeError> {
        Ok(self.ser.end_elem(self.elems.pop().unwrap())?)
    }

    /// Write the HTML *string* to the writer.
    pub fn write_html(&mut self, html: &str) -> Result<(), SerializeError> {
        let sink = SerializerSink {
            ser: Ok(&mut self.ser),
        };
        let mut queue = h5::BufferQueue::new();
        queue.push_back(html.into());
        let mut tokenizer = h5::Tokenizer::new(sink, Default::default());
        let _ = tokenizer.feed(&mut queue);
        tokenizer.sink.ser?;
        Ok(())
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

struct SerializerSink<'s, W: Write> {
    ser: Result<&'s mut h5::HtmlSerializer<W>, SerializeError>,
}

impl<'s, W: Write> SerializerSink<'s, W> {
    fn err(&mut self, err: io::Result<()>) {
        if let Err(err) = err {
            self.ser = Err(SerializeError::Io(err));
        }
    }

    fn start_elem<'a, AttrIter>(&mut self, name: h5::QualName, attrs: AttrIter)
    where
        AttrIter: Iterator<Item = (&'a h5::QualName, &'a str)>,
    {
        if let Ok(ser) = self.ser.as_mut() {
            let err = ser.start_elem(name, attrs);
            self.err(err);
        }
    }

    fn end_elem(&mut self, name: h5::QualName) {
        if let Ok(ser) = self.ser.as_mut() {
            let err = ser.end_elem(name);
            self.err(err);
        }
    }

    fn write_text(&mut self, text: &str) {
        if let Ok(ser) = self.ser.as_mut() {
            let err = ser.write_text(text);
            self.err(err);
        }
    }

    fn write_comment(&mut self, text: &str) {
        if let Ok(ser) = self.ser.as_mut() {
            let err = ser.write_comment(text);
            self.err(err);
        }
    }

    fn write_doctype(&mut self, name: &str) {
        if let Ok(ser) = self.ser.as_mut() {
            let err = ser.write_doctype(name);
            self.err(err);
        }
    }
}

impl<'s, W: Write> h5::TokenSink for SerializerSink<'s, W> {
    type Handle = ();

    fn process_token(
        &mut self,
        token: h5::Token,
        _line_number: u64,
    ) -> h5::TokenSinkResult<Self::Handle> {
        match token {
            h5::Token::DoctypeToken(doctype) => {
                let mut name_str = String::new();
                if let Some(name) = doctype.name {
                    name_str.push_str(&name.as_str_lossy());
                }
                if let Some(public_id) = doctype.public_id {
                    name_str.push_str(&public_id.as_str_lossy());
                }
                if let Some(system_id) = doctype.system_id {
                    name_str.push_str(&system_id.as_str_lossy());
                }
                self.write_doctype(&name_str);
            }
            h5::Token::TagToken(tag) => match tag.kind {
                h5::TagKind::StartTag => {
                    let attrs: Vec<_> = tag
                        .attrs
                        .iter()
                        .map(|attr| (attr.name.clone(), attr.value.as_str_lossy()))
                        .collect();
                    self.start_elem(
                        h5::QualName::new(None, ns!(html), tag.name),
                        attrs.iter().map(|(name, val)| (name, val.borrow())),
                    );
                }
                h5::TagKind::EndTag => {
                    self.end_elem(h5::QualName::new(None, ns!(html), tag.name));
                }
            },
            h5::Token::CommentToken(s) => self.write_comment(&s.as_str_lossy()),
            h5::Token::CharacterTokens(s) => self.write_text(&s.as_str_lossy()),
            h5::Token::NullCharacterToken => {}
            h5::Token::EOFToken => {}
            h5::Token::ParseError(e) => {
                self.ser = Err(SerializeError::ParseError(e));
            }
        }
        h5::TokenSinkResult::Continue
    }
}

/// Errors caused when serializing HTML
#[derive(Debug, Error)]
pub enum SerializeError {
    /// Write error.
    #[error("{0}")]
    Io(#[from] io::Error),

    /// Missing parent element.
    #[error("Attempted to end an element before opening it.")]
    EndEmpty,

    /// Error when parsing HTML.
    #[error("Parse error: {0}")]
    ParseError(Cow<'static, str>),
}
