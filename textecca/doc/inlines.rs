use super::Length;
use super::{Inlines, Meta};

/// A group of inlines tagged with some metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct TaggedInlines {
    content: Inlines,
    meta: Meta,
}

/// A link, either to something within this document or to an external URL.
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    /// The link text, if any; if no text is given, the serializer may compute
    /// its own representation. This is likely appropriate for intra-document links.
    text: Option<Inlines>,
    /// The link's label, if any. This may be used for accessibility purposes.
    label: Option<String>,
    /// The link's target.
    target: LinkTarget,
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

// TODO: Support for citations?

/// An inline quotation.
#[derive(Debug, Clone, PartialEq)]
pub struct Quote {
    kind: QuoteKind,
}

/// Quotation markers; see [Wikipedia](https://en.wikipedia.org/wiki/Quotation_mark).
#[derive(Debug, Clone, PartialEq)]
pub enum QuoteKind {
    /// Primary quotes, locale-defined.
    ///
    /// In US English, these are `“…”`.
    ///
    Primary,
    /// Secondary quotes, locale-defined.
    ///
    /// In US English, these are `‘…’`.
    Secondary,
    /// Custom quotation markers. These can be guillemets, CJK brackets, or
    /// anything else.
    Other(Box<Inlines>, Box<Inlines>),
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
