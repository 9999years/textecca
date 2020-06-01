use crate::doc::{Inline, InlineCode, InlineMath, Inlines, Quote, QuoteKind};
use crate::parse::parse_util as pu;

struct Slugify(String);

impl Slugify {
    fn str(&mut self, text: &str) {
        self.0.reserve(text.len());
        for c in text.chars() {
            if pu::is_inline_space(c) || c.is_ascii_control() {
                self.0.push('-');
            } else {
                self.0.push(c);
            }
        }
    }

    fn inline(&mut self, inline: &Inline) {
        match inline {
            Inline::Text(content) | Inline::Code(InlineCode { content, .. }) => {
                self.str(&content);
            }
            Inline::Styled { content, .. } => {
                self.inlines(content);
            }
            Inline::Quote(Quote { content, kind }) => match kind {
                QuoteKind::Primary => {
                    // TODO: Support locale-dependent quotes
                    self.0.push('“');
                    self.inlines(content);
                    self.0.push('”');
                }
                QuoteKind::Secondary => {
                    self.0.push('‘');
                    self.inlines(content);
                    self.0.push('’');
                }
                QuoteKind::Other(l, r) => {
                    self.inlines(l);
                    self.inlines(content);
                    self.inlines(r);
                }
            },
            Inline::Space => {
                self.0.push('-');
            }
            Inline::Link(link) => self.inlines(&link.text()),
            Inline::Footnote(_) => {}
            Inline::Math(InlineMath { tex }) => {
                // (big shrug)
                self.str(tex);
            }
        }
    }

    fn inlines(&mut self, inlines: &[Inline]) {
        for inline in inlines {
            self.inline(inline);
        }
    }
}

/// Slugify the given inlines.
pub fn slugify(inlines: &Inlines) -> String {
    let mut ret = Slugify(String::new());
    ret.inlines(inlines);
    ret.0
}
