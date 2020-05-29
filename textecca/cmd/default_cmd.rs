use std::rc::Rc;

use super::{
    Command, CommandError, CommandInfo, FromArgs, FromArgsError, ParsedArgs, Thunk, World,
};
use crate::doc::Blocks;
use crate::env::Environment;
use crate::parse::{Parser, Tokens};

const DEFAULT_COMMAND_NAME: &str = "\u{e70a}default";

pub struct DefaultCommand<'i> {
    doc: Thunk<'i>,
}

impl<'i> DefaultCommand<'i> {
    fn from_args<'a>(
        parsed: &mut ParsedArgs<'a>,
    ) -> Result<Box<dyn Command<'a> + 'a>, FromArgsError> {
        let doc = parsed.args.pop().ok_or(FromArgsError::NotEnough)?;
        if parsed.args.len() != 1 {
            return Err(FromArgsError::TooMany);
        }
        if !parsed.kwargs.is_empty() {
            return Err(FromArgsError::from_extra_kwargs(parsed));
        }

        Ok(Box::new(DefaultCommand { doc }))
    }

    pub fn info() -> CommandInfo {
        CommandInfo::from_name_and_args(DEFAULT_COMMAND_NAME.to_owned(), Self::from_args)
    }
}

impl<'i> Command<'i> for DefaultCommand<'i> {
    fn call(self: Box<Self>, world: &World<'i>) -> Result<Blocks, CommandError> {
        Ok(self.doc.force(world)?)
    }
}
