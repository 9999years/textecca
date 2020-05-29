use std::rc::Rc;

use super::{CommandError, ParsedArgs, World};
use crate::doc::{Block, Blocks, Inline};
use crate::env::Environment;
use crate::parse::{Source, Token, Tokens};

#[derive(Debug, Clone, PartialEq)]
pub enum Thunk<'i> {
    Lazy(Tokens<'i>),
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
    pub fn force(self, world: &World<'i>) -> Result<Blocks, CommandError> {
        match self {
            Self::Lazy(tokens) => {
                let mut ret = Vec::with_capacity(tokens.len());
                for tok in tokens {
                    match tok {
                        Token::Text(sp) => {
                            ret.push(Block::Plain(vec![Inline::Text(sp.to_string())]));
                        }
                        Token::Command(cmd) => {
                            let name = *cmd.name.fragment();
                            let info = world
                                .env
                                .cmd_info(name)
                                .ok_or_else(|| CommandError::Name(name.to_string()))?;
                            let mut args =
                                ParsedArgs::from_unparsed(&cmd.args, info.parser_fn, world)
                                    .map_err(CommandError::ParseError)?;
                            let cmd = (info.from_args_fn)(&mut args)?;
                            ret.append(&mut cmd.call(world)?);
                        }
                    }
                }
                Ok(ret)
            }
            Self::Forced(blocks) => Ok(blocks),
        }
    }
}
