use std::rc::Rc;

use super::{
    Command, CommandError, CommandInfo, FromArgs, FromArgsError, ParsedArgs, Thunk, World,
};
use crate::doc::Blocks;
use crate::env::Environment;
use crate::parse::{Parser, Tokens};

const DEFAULT_COMMAND_NAME: &str = "__default__";

#[derive(Debug)]
pub struct DefaultCommand<'i> {
    doc: Thunk<'i>,
}

impl<'i> DefaultCommand<'i> {
    fn from_args<'a>(
        parsed: &mut ParsedArgs<'a>,
    ) -> Result<Box<dyn Command<'a> + 'a>, FromArgsError> {
        let doc = parsed.pop_positional()?;
        parsed.check_no_args()?;
        Ok(Box::new(DefaultCommand { doc }))
    }

    pub fn info() -> CommandInfo {
        CommandInfo::from_name_and_args(DEFAULT_COMMAND_NAME.to_owned(), Self::from_args)
    }
}

impl<'i> Command<'i> for DefaultCommand<'i> {
    fn call(self: Box<Self>, world: &World<'i>) -> Result<Blocks, CommandError<'i>> {
        Ok(self.doc.force(world)?)
    }
}
