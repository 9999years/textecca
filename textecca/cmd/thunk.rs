use std::{convert::TryInto, rc::Rc};

use super::{CommandError, ParsedArgs, World};
use crate::doc::{BlockInner, Blocks, DocBuilder, DocBuilderPush, Inline, Inlines};
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
    /// Evaluate this thunk if it's `Lazy`, otherwise, write its `Blocks` to the given `DocBuilder`.
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

    /// Evaluate the given `Thunk` and return its blocks directly; avoids
    /// manually creating a temporary `DocBuilder`.
    pub fn into_blocks(self, world: &World<'i>) -> Result<Blocks, CommandError<'i>> {
        let mut doc = DocBuilder::new();
        self.force(world, &mut doc)?;
        Ok(doc.try_into()?)
    }

    /// Evaluate the given `Thunk` and extract its inlines; errors if the `Thunk` renders to `Blocks`.
    pub fn into_inlines(self, world: &World<'i>) -> Result<Inlines, CommandError<'i>> {
        let mut doc = DocBuilder::new();
        self.force(world, &mut doc)?;
        Ok(doc.try_into()?)
    }

    /// Render this `Thunk` as a string if it's `Lazy`, and give an error if it's
    /// `Forced` or contains `Command` tokens.
    pub fn into_string(&self) -> Result<String, CommandError<'i>> {
        match self {
            Thunk::Lazy(toks) => {
                let mut ret = String::with_capacity(toks.len() * 16);
                for tok in toks {
                    match tok {
                        Token::Text(span) => {
                            ret.push_str(span.fragment());
                        }
                        Token::Command(_) => return Err(CommandError::BadToken(tok.clone())),
                    }
                }
                Ok(ret)
            }
            Thunk::Forced(_) => Err(CommandError::ForcedThunk),
        }
    }
}
