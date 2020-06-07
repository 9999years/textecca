use std::io::{self, Write};
use std::iter;
use std::mem;
use std::{borrow::Cow, vec};

use thiserror::Error;
use typed_html::html;

use friendly_html as fh;

use super::{InitSerializer, Serializer, SerializerError};
use crate::doc::{
    self, Block, BlockInner, Blocks, Doc, Footnote, Heading, Inline, Inlines, List, ListKind,
};

mod math;
mod slugify;

use math::*;
pub use slugify::*;

/// Serializer to HTML5.
pub struct HtmlSerializer<W: Write> {
    ser: fh::HtmlSerializer<W>,
    footnotes: Vec<MarkedFootnote>,
}

struct MarkedFootnote {
    id: String,
    return_id: String,
    content: Blocks,
}

impl<W: Write> InitSerializer<W> for HtmlSerializer<W> {
    fn new(writer: W) -> Result<Box<Self>, SerializerError> {
        Ok(Box::new(Self {
            ser: fh::HtmlSerializer::with_doctype(writer)?,
            footnotes: Default::default(),
        }))
    }
}

impl<W: Write> Serializer for HtmlSerializer<W> {
    fn write_doc(&mut self, doc: Doc) -> Result<(), SerializerError> {
        self.write_header(&doc)?;
        self.write_blocks(doc.content)?;
        self.finish_footnotes()?;
        self.finish()?;
        Ok(())
    }
}

impl<W: Write> HtmlSerializer<W> {
    fn write_header(&mut self, doc: &Doc) -> Result<(), SerializerError> {
        self.ser.elem("html")?;
        self.ser.write_text("\n")?;
        self.ser.elem("head")?;
        self.ser.write_text("\n")?;
        if doc.has_math() {
            // TODO: Don't hardcode KaTeX tag.
            self.ser.elem_attrs(
                "link",
                &[
                    ("rel", "stylesheet"),
                    (
                        "href",
                        "https://cdn.jsdelivr.net/npm/katex@0.11.1/dist/katex.min.css",
                    ),
                    (
                        "integrity",
                        "sha384-zB1R0rpPzHqg7Kpt0Aljp8JPLqbXI3bhnPWROx27a9N0Ll6ZP/+DiW/UqRcLbRjq",
                    ),
                    ("crossorigin", "anonymous"),
                ],
            )?;
            self.ser.write_text("\n")?;
        }
        self.ser.end_elem()?;
        self.ser.write_text("\n")?;
        self.ser.elem("body")?;
        self.ser.write_text("\n")?;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), SerializerError> {
        self.ser.write_text("\n")?;
        self.ser.end_elem()?; // </body>
        self.ser.write_text("\n")?;
        self.ser.end_elem()?; // </html>
        self.ser.write_text("\n")?;
        Ok(())
    }

    fn write_styled(
        &mut self,
        style: &doc::Style,
        content: &Inlines,
    ) -> Result<(), SerializerError> {
        match style {
            doc::Style::Emph => {
                self.ser.elem("em")?;
                self.write_inlines(content)?;
                self.ser.end_elem()?;
            }
            doc::Style::Strong => {
                self.ser.elem("strong")?;
                self.write_inlines(content)?;
                self.ser.end_elem()?;
            }
            _ => todo!("Unimplemented style {:?}", style),
        }
        Ok(())
    }

    fn write_inlines(&mut self, inlines: &[Inline]) -> Result<(), SerializerError> {
        for inline in inlines {
            self.write_inline(Cow::Borrowed(inline))?;
        }
        Ok(())
    }

    fn write_inline(&mut self, inline: Cow<Inline>) -> Result<(), SerializerError> {
        match inline.as_ref() {
            Inline::Text(content) => {
                self.ser.write_text(content)?;
            }
            Inline::Styled { style, content } => self.write_styled(&style, &content)?,
            Inline::Quote(quote) => {
                let (l, r) = quote.kind.to_inlines();
                self.write_inlines(&l)?;
                self.write_inlines(&quote.content)?;
                self.write_inlines(&r)?;
            }
            Inline::Code(code) => {
                if let Some(lang) = &code.language {
                    self.ser.elem_attrs("code", &[("class", &lang)])?;
                } else {
                    self.ser.elem("code")?;
                }
                self.ser.write_text(&code.content)?;
                self.ser.end_elem()?;
            }
            Inline::Space => {
                self.ser.write_text(" ")?;
            }
            Inline::Link(_) => {}
            Inline::Footnote(_) => match inline.into_owned() {
                Inline::Footnote(footnote) => self.write_footnote(footnote)?,
                _ => unreachable!(),
            },
            Inline::Math(math) => {
                self.ser
                    .write_html(&render_tex(&math.tex, MathMode::Inline)?)?;
            }
        }
        Ok(())
    }

    fn write_list(&mut self, list: List) -> Result<(), SerializerError> {
        let list_tag = match list.kind {
            ListKind::Unordered => "ul",
            ListKind::Ordered => "ol",
        };
        self.ser.elem(list_tag)?;
        for item in list.items {
            self.ser.elem("li")?;
            self.write_blocks(item.content)?;
            self.ser.end_elem()?;
        }
        self.ser.end_elem()?;
        Ok(())
    }

    fn write_blocks(&mut self, blocks: Blocks) -> Result<(), SerializerError> {
        for block in blocks {
            self.write_block(block)?;
        }
        Ok(())
    }

    fn write_block(&mut self, block: Block) -> Result<(), SerializerError> {
        match block.inner {
            BlockInner::Plain(inlines) => {
                self.write_inlines(&inlines)?;
            }
            BlockInner::Par(inlines) => {
                self.ser.write_text("\n")?;
                self.ser.elem("p")?;
                self.write_inlines(&inlines)?;
                self.ser.end_elem()?;
            }
            BlockInner::Code(_) => todo!(),
            BlockInner::Quote(quote) => {
                self.ser.elem("blockquote")?;
                self.write_blocks(quote)?;
                self.ser.end_elem()?;
            }
            BlockInner::List(list) => self.write_list(list)?,
            BlockInner::Heading(heading) => {
                if !(1..6).contains(&heading.level) {
                    return Err(HtmlError::from(heading).into());
                }
                let tag_name = format!("h{}", heading.level);
                let slug = slugify(&heading.text);
                self.ser.elem_attrs(&tag_name, &[("id", &slug)])?;

                self.ser
                    .elem_attrs("a", &[("href", format!("#{}", &slug))])?;
                self.ser.end_elem()?;

                self.write_inlines(&heading.text)?;

                self.ser.end_elem()?;
            }
            BlockInner::Rule => {
                self.ser.elem("hr")?;
            }
            BlockInner::Math(math) => {
                self.ser
                    .write_html(&render_tex(&math.tex, MathMode::Display)?)?;
            }
            BlockInner::Table(_) => todo!(),
            BlockInner::Figure(_) => todo!(),
            BlockInner::Defn(_) => todo!(),
            BlockInner::TermList(_) => todo!(),
        }
        Ok(())
    }

    fn write_footnote(&mut self, footnote: Footnote) -> Result<(), SerializerError> {
        let num = self.footnotes.len() + 1;
        let id = format!("fn-{}", num);
        let return_id = format!("fn-link-{}", num);
        self.ser.elem("sup")?;
        self.ser
            .elem_attrs("a", &[("href", &format!("#{}", &id)), ("id", &return_id)])?;
        self.ser.write_text(format!("[{}]", num))?;
        self.ser.end_elem()?; // </a>
        self.ser.end_elem()?; // </sup>
        self.footnotes.push(MarkedFootnote {
            id,
            return_id,
            content: footnote.content,
        });
        Ok(())
    }

    fn finish_footnote(&mut self, footnote: MarkedFootnote) -> Result<(), SerializerError> {
        // TODO: Write self-link.
        self.write_blocks(footnote.content)?;
        self.ser.write_text(" ")?;
        self.ser
            .elem_attrs("a", &[("href", format!("#{}", footnote.return_id))])?;
        self.ser.write_text("↩")?;
        self.ser.end_elem()?;
        Ok(())
    }

    fn finish_footnotes(&mut self) -> Result<(), SerializerError> {
        if self.footnotes.is_empty() {
            return Ok(());
        }

        self.ser.elem_attrs("ol", &[("class", "footnotes")])?;
        for footnote in mem::take(&mut self.footnotes) {
            self.ser.elem_attrs("li", &[("id", &footnote.id)])?;
            self.finish_footnote(footnote)?;
            self.ser.end_elem()?;
        }
        self.ser.end_elem()?;
        Ok(())
    }
}

/// An error when serializing HTML.
#[derive(Debug, Error)]
pub enum HtmlError {
    /// A bad document heading, in particular an unsupported level.
    #[error("Bad heading: {0:?}")]
    BadHeading(Heading),
}

impl From<Heading> for HtmlError {
    fn from(h: Heading) -> Self {
        Self::BadHeading(h)
    }
}

impl Into<SerializerError> for HtmlError {
    fn into(self) -> SerializerError {
        SerializerError::Other(Box::new(self))
    }
}

impl From<fh::SerializeError> for SerializerError {
    fn from(err: fh::SerializeError) -> Self {
        match err {
            fh::SerializeError::Io(e) => SerializerError::Io(e),
            _ => SerializerError::Other(Box::new(err)),
        }
    }
}
