#![allow(missing_docs)] // TODO: Remove this?

use crate::{
    cmd::{Command, CommandError, CommandInfo, FromArgs, FromArgsError, ParsedArgs, Thunk, World},
    doc::{Block, Blocks, DocBuilder, DocBuilderPush as _, Heading},
    env::Environment,
};

/// Adds the builtins bindings to the given `Environment`.
pub fn import(env: &mut Environment) {
    env.add_binding::<Par>();
    env.add_binding::<Sec>();
}

macro_rules! cmd_info {
    {$cmd:ty; $name:literal; fn from_args($args:ident) $from_args:block} => {
        impl $cmd {
            fn from_args<'a>(
                $args: &mut ParsedArgs<'a>,
            ) -> Result<Box<dyn Command<'a> + 'a>, FromArgsError>
            $from_args
        }

        impl CommandInfo for $cmd {
            fn name() -> String {
                String::from($name)
            }

            fn from_args_fn() -> FromArgs {
                Self::from_args
            }
        }
    };
}

#[derive(Debug)]
pub struct Par {}

cmd_info! {
    Par;
    "par";
    fn from_args(parsed) {
        parsed.check_no_args()?;
        Ok(Box::new(Par {}))
    }
}

impl<'i> Command<'i> for Par {
    fn call(
        self: Box<Self>,
        doc: &mut DocBuilder,
        _world: &World<'i>,
    ) -> Result<(), CommandError<'i>> {
        doc.push(Block::Par(Default::default()))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Sec<'i> {
    title: Thunk<'i>,
}
impl<'i> Sec<'i> {
    fn from_args<'a>(
        parsed: &mut ParsedArgs<'a>,
    ) -> Result<Box<dyn Command<'a> + 'a>, FromArgsError> {
        let title = parsed.pop_positional()?;
        parsed.check_no_args()?;
        Ok(Box::new(Sec { title }))
    }
}

impl<'i> CommandInfo for Sec<'i> {
    fn name() -> String {
        String::from("sec")
    }

    fn from_args_fn() -> FromArgs {
        Self::from_args
    }
}

impl<'i> Command<'i> for Sec<'i> {
    fn call(
        self: Box<Self>,
        doc: &mut DocBuilder,
        world: &World<'i>,
    ) -> Result<(), CommandError<'i>> {
        doc.push(Block::Heading(Heading {
            level: 1,
            text: Default::default(),
        }))?;
        self.title.force(world, doc)?;
        Ok(())
    }
}
