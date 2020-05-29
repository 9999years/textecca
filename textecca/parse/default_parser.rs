use super::{Command, Parser, RawToken, RawTokens, Source, Token, Tokens};

/// The default textecca parser.
pub fn default_parser<'i>(arena: &'i Source, raw_tokens: RawTokens<'i>) -> Tokens<'i> {
    let mut ret = Vec::with_capacity(raw_tokens.len());
    for tok in raw_tokens {
        match tok {
            RawToken::Line(line) => {
                ret.push(Token::Text(line.text));
                ret.push(Token::Text(line.newline));
            }
            RawToken::BlankLines(blanklines) => {
                ret.push(
                    Command::from_name(arena.alloc_span("par".to_owned(), blanklines.span)).into(),
                );
            }
        }
    }
    ret
}
