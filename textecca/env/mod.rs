//! Evaluation environment, binding names to commands.

use std::collections::HashMap;
use std::rc::Rc;

use derivative::Derivative;

use crate::cmd::{Command, CommandError, CommandInfo, FromArgs, FromArgsError, ParsedArgs, World};
use crate::parse::{self, Parser};

/// An evaluation environment, mapping command names to bindings and inheriting
/// from a parent environment.
#[derive(Default, Debug, Clone)]
pub struct Environment {
    parent: Option<Rc<Environment>>,
    cmds: HashMap<String, CommandInfo>,
}

impl Environment {
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

    pub fn cmd_info(&self, name: &str) -> Result<&CommandInfo, CommandError<'static>> {
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

    pub fn add_binding(&mut self, cmd_info: CommandInfo) {
        self.cmds.insert(cmd_info.name.clone(), cmd_info);
    }
}
