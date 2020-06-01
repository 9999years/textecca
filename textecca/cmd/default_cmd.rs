use std::rc::Rc;

use super::{
    Command, CommandError, CommandInfo, FromArgs, FromArgsError, ParsedArgs, Thunk, World,
};
use crate::doc::Blocks;
use crate::parse::Parser;

const DEFAULT_COMMAND_NAME: &str = "__default__";

/// The "default command"; essentially specifies the parser to be used at the top-level.
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
}

impl<'i> CommandInfo for DefaultCommand<'i> {
    fn name() -> String {
        DEFAULT_COMMAND_NAME.to_owned()
    }

    fn from_args_fn() -> FromArgs {
        Self::from_args
    }
}

impl<'i> Command<'i> for DefaultCommand<'i> {
    fn call(
        self: Box<Self>,
        doc: &mut crate::doc::DocBuilder,
        world: &World<'i>,
    ) -> Result<(), CommandError<'i>> {
        self.doc.force(world, doc)
    }
}
