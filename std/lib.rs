#![allow(missing_docs)] // TODO: Remove this?
use std::error;

use derive_command::CommandInfo;

use textecca::{
    cmd::{Command, CommandError, CommandInfo, Thunk, World},
    doc::{self, Block, DocBuilder, DocBuilderPush as _, Heading, Inline},
    env::Environment,
    parse::{Source, Span, Token, Tokens},
};

/// Adds the builtins bindings to the given `Environment`.
pub fn import(env: &mut Environment) {
    env.add_binding::<Par>();
    env.add_binding::<Sec>();
    env.add_binding::<Footnote>();
    env.add_binding::<Code>();
    env.add_binding::<Emph>();
    env.add_binding::<Strong>();
    env.add_binding::<Math>();
}

fn literal_parser<'i>(
    _arena: &'i Source,
    input: Span<'i>,
) -> Result<Tokens<'i>, Box<dyn error::Error + 'i>> {
    Ok(vec![Token::Text(input)])
}

#[derive(Debug, CommandInfo)]
pub struct Par {}
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

#[derive(Debug, CommandInfo)]
pub struct Sec<'i> {
    title: Thunk<'i>,
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

#[derive(Debug, CommandInfo)]
pub struct Footnote<'i> {
    content: Thunk<'i>,
}
impl<'i> Command<'i> for Footnote<'i> {
    fn call(
        self: Box<Self>,
        doc: &mut DocBuilder,
        world: &World<'i>,
    ) -> Result<(), CommandError<'i>> {
        doc.push(Inline::Footnote(doc::Footnote {
            content: self.content.into_blocks(world)?,
        }))?;
        Ok(())
    }
}

#[derive(Debug, CommandInfo)]
#[textecca(parser = literal_parser)]
pub struct Code<'i> {
    content: Thunk<'i>,
}
impl<'i> Command<'i> for Code<'i> {
    fn call(
        self: Box<Self>,
        doc: &mut DocBuilder,
        _world: &World<'i>,
    ) -> Result<(), CommandError<'i>> {
        doc.push(Inline::Code(doc::InlineCode {
            language: None,
            content: self.content.into_string()?,
        }))?;
        Ok(())
    }
}

#[derive(Debug, CommandInfo)]
pub struct Emph<'i> {
    content: Thunk<'i>,
}
impl<'i> Command<'i> for Emph<'i> {
    fn call(
        self: Box<Self>,
        doc: &mut DocBuilder,
        world: &World<'i>,
    ) -> Result<(), CommandError<'i>> {
        doc.push(Inline::Styled {
            style: doc::Style::Emph,
            content: self.content.into_inlines(world)?,
        })?;
        Ok(())
    }
}

#[derive(Debug, CommandInfo)]
pub struct Strong<'i> {
    content: Thunk<'i>,
}
impl<'i> Command<'i> for Strong<'i> {
    fn call(
        self: Box<Self>,
        doc: &mut DocBuilder,
        world: &World<'i>,
    ) -> Result<(), CommandError<'i>> {
        doc.push(Inline::Styled {
            style: doc::Style::Strong,
            content: self.content.into_inlines(world)?,
        })?;
        Ok(())
    }
}

#[derive(Debug, CommandInfo)]
#[textecca(parser = literal_parser)]
pub struct Math<'i> {
    content: Thunk<'i>,
}
impl<'i> Command<'i> for Math<'i> {
    fn call(
        self: Box<Self>,
        doc: &mut DocBuilder,
        _world: &World<'i>,
    ) -> Result<(), CommandError<'i>> {
        doc.push(Inline::Math(doc::InlineMath {
            tex: self.content.into_string()?,
        }))?;
        Ok(())
    }
}
