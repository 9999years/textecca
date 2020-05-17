use crate::lex::{
    tokenize::{Token, Tokens},
    Span,
};

#[derive(Clone, Debug, PartialEq)]
pub enum BlockChild<'i> {
    Token(Token<'i>),
    Block(Block<'i>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block<'i> {
    indent: Span<'i>,
    contents: Vec<BlockChild<'i>>,
}

impl<'i> Block<'i> {
    pub fn new(indent: Span<'i>, contents: Vec<BlockChild<'i>>) -> Self {
        Block { indent, contents }
    }
}

impl<'i> From<Tokens<'i>> for Block<'i> {
    fn from(tokens: Tokens<'i>) -> Self {
        // Simple stack-based block parser.
        // Remaining input tokens, used for Vec capacity.
        let mut remaining_toks = tokens.toks.len();
        // The block currently being parsed.
        let mut current: Block = Block {
            indent: Span::new(""),
            contents: Vec::with_capacity(remaining_toks),
        };
        // The stack of surrounding blocks.
        let mut stack: Vec<Block> = Vec::new();
        for tok in tokens.toks {
            match tok {
                Token::Indent(indent) => {
                    // Indentation; save the partially-parsed current block to
                    // the stack and begin parsing this new block.
                    remaining_toks -= current.contents.len();
                    stack.push(current);
                    current = Block {
                        indent,
                        contents: Vec::with_capacity(remaining_toks),
                    };
                }
                Token::Deindent(n) => {
                    // Pop and finalize *n* items off the stack.
                    for _ in 0..n {
                        // If we're wasting more than 50% (?) of the vector's
                        // capacity, shrink to fit.
                        if current.contents.capacity()
                            > (current.contents.len() as f64 * 1.5) as usize
                        {
                            current.contents.shrink_to_fit();
                        }
                        let mut new_current = stack.pop().unwrap();
                        new_current.contents.push(BlockChild::Block(current));
                        current = new_current;
                    }
                }
                _ => {
                    current.contents.push(BlockChild::Token(tok));
                }
            }
        }
        current
    }
}

#[cfg(test)]
mod test {
    use nom::error::VerboseError;

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::lex::test_util::Input;
    use crate::lex::tokenize::tokenize;

    #[test]
    fn block_from_tokens() {
        let input = Input::new(indoc!(
            r#"
            noIndent
                extraIndent
                    extraIndentAgain
                deindentOne
                sameIndent
                        extraIndent
            deindentTwo
            sameIndent
            "#
        ));
        assert_eq!(
            Block::new(
                Span::new(""),
                vec![
                    BlockChild::Token(Token::Word(input.offset(0, "noIndent"))),
                    BlockChild::Token(Token::Newline(input.offset(8, "\n"))),
                    BlockChild::Block(Block::new(
                        input.offset(9, "    "),
                        vec![
                            BlockChild::Token(Token::Word(input.offset(13, "extraIndent"))),
                            BlockChild::Token(Token::Newline(input.offset(24, "\n"))),
                            BlockChild::Block(Block::new(
                                input.offset(29, "    "),
                                vec![
                                    BlockChild::Token(Token::Word(
                                        input.offset(33, "extraIndentAgain")
                                    )),
                                    BlockChild::Token(Token::Newline(input.offset(49, "\n"))),
                                ]
                            )),
                            BlockChild::Token(Token::Word(input.offset(54, "deindentOne"))),
                            BlockChild::Token(Token::Newline(input.offset(65, "\n"))),
                            BlockChild::Token(Token::Word(input.offset(70, "sameIndent"))),
                            BlockChild::Token(Token::Newline(input.offset(80, "\n"))),
                            BlockChild::Block(Block::new(
                                input.offset(85, "        "),
                                vec![
                                    BlockChild::Token(Token::Word(input.offset(93, "extraIndent"))),
                                    BlockChild::Token(Token::Newline(input.offset(104, "\n"))),
                                ]
                            )),
                        ]
                    )),
                    BlockChild::Token(Token::Word(input.offset(105, "deindentTwo"))),
                    BlockChild::Token(Token::Newline(input.offset(116, "\n"))),
                    BlockChild::Token(Token::Word(input.offset(117, "sameIndent"))),
                    BlockChild::Token(Token::Newline(input.offset(127, "\n"))),
                ]
            ),
            tokenize::<VerboseError<_>>(input.as_span()).unwrap().into()
        );
    }
}
