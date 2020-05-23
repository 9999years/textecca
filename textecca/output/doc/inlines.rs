use super::super::length::Length;
use super::structure::Inlines;

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    text: Inlines,
    label: Option<String>,
    target: String,
}

/// Maybe at some point in the future.
#[derive(Debug, Clone, PartialEq)]
pub struct Citation {}

#[derive(Debug, Clone, PartialEq)]
pub struct Quote {
    kind: QuoteKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuoteKind {
    Single,
    Double,
    /// Left, right quote markers.
    Other(Box<Inlines>, Box<Inlines>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Style {
    Emph,
    Strong,
    Superscript,
    Subscript,
    SmallCaps,
    Strikeout,
    Underline,
    Size(Length),
    Color(Color),
    Font(Font),
    FontFeatures(FontFeatures),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Color {}
#[derive(Debug, Clone, PartialEq)]
pub struct Font {}
#[derive(Debug, Clone, PartialEq)]
pub struct FontFeatures {}
