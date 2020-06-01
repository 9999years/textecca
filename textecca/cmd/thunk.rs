use std::rc::Rc;

use super::{CommandError, ParsedArgs, World};
use crate::doc::{Block, Blocks, DocBuilder, DocBuilderPush, Inline};
use crate::env::Environment;
use crate::parse::{Source, Token, Tokens};

/// A lazily-evaluated `Command` argument.
#[derive(Debug, Clone, PartialEq)]
pub enum Thunk<'i> {
    /// An unevaluated sequence of `Tokens`.
    Lazy(Tokens<'i>),
    /// An evaluated sequence of `Blocks`.
    Forced(Blocks),
}

impl<'i> From<Tokens<'i>> for Thunk<'i> {
    fn from(tokens: Tokens<'i>) -> Self {
        Self::Lazy(tokens)
    }
}

impl<'i> From<Blocks> for Thunk<'i> {
    fn from(blocks: Blocks) -> Self {
        Self::Forced(blocks)
    }
}

impl<'i> Thunk<'i> {
    /// Evaluate this thunk if it's `Lazy`, otherwise, return its `Blocks`.
    pub fn force(self, world: &World<'i>, doc: &mut DocBuilder) -> Result<(), CommandError<'i>> {
        match self {
            Self::Lazy(tokens) => {
                for tok in tokens {
                    match tok {
                        Token::Text(sp) => {
                            doc.push(sp)?;
                        }
                        Token::Command(cmd) => {
                            world.call_cmd(cmd, doc)?;
                        }
                    }
                }
                Ok(())
            }
            Self::Forced(blocks) => {
                doc.push(blocks)?;
                Ok(())
            }
        }
    }
}
