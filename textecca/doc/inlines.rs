use super::Length;
use super::{Blocks, Inline, Inlines, Meta};
use std::borrow::Cow;

/// A group of inlines tagged with some metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct TaggedInlines {
    /// The contained text.
    pub content: Inlines,
    /// The tagged metadata.
    pub meta: Meta,
}

/// A link, either to something within this document or to an external URL.
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    /// The link text, if any; if no text is given, the serializer may compute
    /// its own representation. This is likely appropriate for intra-document links.
    pub content: Option<Inlines>,
    /// The link's label, if any. This may be used for accessibility purposes.
    pub label: Option<String>,
    /// The link's target.
    pub target: LinkTarget,
}

impl Link {
    /// Get the link's text, from `text`, `label`, or `target` (in that order).
    pub fn text(&self) -> Cow<[Inline]> {
        if let Some(inlines) = &self.content {
            Cow::Borrowed(inlines)
        } else if let Some(text) = &self.label {
            Cow::Owned(vec![Inline::Text(text.into())])
        } else {
            Cow::Owned(vec![Inline::Text(self.target.as_str().into())])
        }
    }
}

/// A `Link`'s destination, either within the document or external.
#[derive(Debug, Clone, PartialEq)]
pub enum LinkTarget {
    /// A label defined elsewhere in the document. If the label is never defined,
    /// an error is raised.
    Label(String),
    /// A URL.
    URL(String),
}

impl LinkTarget {
    /// Get the target of this link as a string slice.
    pub fn as_str(&self) -> &str {
        match self {
            LinkTarget::Label(s) | LinkTarget::URL(s) => &s,
        }
    }
}

// TODO: Support for citations?

/// An inline quotation.
#[derive(Debug, Clone, PartialEq)]
pub struct Quote {
    /// The quotation markers.
    pub kind: QuoteKind,
    /// The quotation text.
    pub content: Inlines,
}

/// Quotation markers; see [Wikipedia](https://en.wikipedia.org/wiki/Quotation_mark).
#[derive(Debug, Clone, PartialEq)]
pub enum QuoteKind {
    /// Primary quotes, locale-defined.
    ///
    /// In US English, these are `“…”`.
    Primary,
    /// Secondary quotes, locale-defined.
    ///
    /// In US English, these are `‘…’`.
    Secondary,
    /// Custom quotation markers. These can be guillemets, CJK brackets, or
    /// anything else.
    Other(Box<Inlines>, Box<Inlines>),
}

impl QuoteKind {
    /// Gives a pair of the open and close quote markers as `Inlines`.
    pub fn to_inlines(&self) -> (Cow<[Inline]>, Cow<[Inline]>) {
        match self {
            QuoteKind::Primary => (
                Cow::Owned(vec![Inline::Text("“".into())]),
                Cow::Owned(vec![Inline::Text("”".into())]),
            ),
            QuoteKind::Secondary => (
                Cow::Owned(vec![Inline::Text("‘".into())]),
                Cow::Owned(vec![Inline::Text("’".into())]),
            ),
            QuoteKind::Other(l, r) => (Cow::Borrowed(&l), Cow::Borrowed(&r)),
        }
    }
}

/// Styled text.
#[derive(Debug, Clone, PartialEq)]
pub enum Style {
    /// Emphasized text, typically displayed with italics.
    Emph,
    /// Strong text, typically displayed with bold.
    Strong,
    /// Superscript text.
    Superscript,
    /// Subscript text.
    Subscript,
    /// Small-caps text.
    SmallCaps,
    /// Struck-out text.
    Strikeout,
    /// Underlined text.
    Underline,
    /// Text in a given size.
    Size(Length),
    /// Text in a given color.
    Color(Color),
    /// Text in a given font.
    Font(Font),
    /// Text with the given font-features activated.
    FontFeatures(FontFeatures),
}

/// Colored text.
#[derive(Debug, Clone, PartialEq)]
pub struct Color {}

/// Text in a custom font.
#[derive(Debug, Clone, PartialEq)]
pub struct Font {}

/// Text with particular font features activated.
#[derive(Debug, Clone, PartialEq)]
pub struct FontFeatures {}

/// An inline code snippet.
#[derive(Debug, Clone, PartialEq)]
pub struct InlineCode {
    /// The code's language, for highlighting. `"plain"` indicates no highlighting.
    pub language: Option<String>,
    /// The code.
    pub content: String,
}

/// A footnote.
#[derive(Debug, Clone, PartialEq)]
pub struct Footnote {
    /// The footnote text.
    pub content: Blocks,
}

/// Inline mathematical text.
#[derive(Debug, Clone, PartialEq)]
pub struct InlineMath {
    /// The math to render, as `LaTeX`.
    pub tex: String,
}
