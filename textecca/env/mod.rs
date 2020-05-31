//! Evaluation environment, binding names to commands.

use std::collections::HashMap;
use std::rc::Rc;

use derivative::Derivative;

use crate::cmd::{
    Command, CommandError, CommandInfo, CommandInfoMemo, FromArgs, FromArgsError, ParsedArgs, World,
};
use crate::parse::{self, Parser};

/// An evaluation environment, mapping command names to bindings and inheriting
/// from a parent environment.
#[derive(Default, Debug, Clone)]
pub struct Environment {
    parent: Option<Rc<Environment>>,
    cmds: HashMap<String, CommandInfoMemo>,
}

impl Environment {
    /// Create a new environment.
    pub fn new() -> Rc<Self> {
        Rc::new(Default::default())
    }

    /// Creates a new environment inheriting from this one.
    pub fn new_inheriting(self: Rc<Self>) -> Rc<Self> {
        Rc::new(Self {
            parent: Some(self),
            ..Default::default()
        })
    }

    /// Get the memozied information for the command with the given name.
    pub fn cmd_info(&self, name: &str) -> Result<&CommandInfoMemo, CommandError<'static>> {
        self.cmds
            .get(name)
            .or_else(|| {
                self.parent
                    .as_ref()
                    .map(|env| env.cmd_info(name).ok())
                    .flatten()
            })
            .ok_or_else(|| CommandError::Name(name.to_owned()))
    }

    /// Add a binding from the given type.
    pub fn add_binding<C: CommandInfo>(&mut self) {
        let info = CommandInfoMemo::new::<C>();
        self.cmds.insert(info.name.clone(), info);
    }

    /// Add a binding from the given type, but override the binding's name.
    pub fn add_binding_name<C: CommandInfo>(&mut self, name: String) {
        self.cmds.insert(name, CommandInfoMemo::new::<C>());
    }
}
