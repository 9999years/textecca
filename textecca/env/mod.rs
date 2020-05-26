//! Evaluation environment, binding names to commands.

use std::collections::HashMap;
use std::rc::Rc;

use derivative::Derivative;

use crate::cmd::{Command, CommandInfo, FromArgs, FromArgsError, ParsedArgs};
use crate::parse::Parser;

#[derive(Derivative, Clone, Copy)]
#[derivative(Debug)]
struct EnvCommandInfo {
    #[derivative(Debug = "ignore")]
    from_args: FromArgs,
    #[derivative(Debug = "ignore")]
    parser: Parser,
}

#[derive(Default, Debug, Clone)]
pub struct Environment {
    parent: Option<Rc<Environment>>,
    cmds: HashMap<String, EnvCommandInfo>,
}

impl Environment {
    fn cmd_info(&self, name: &str) -> Option<&EnvCommandInfo> {
        self.cmds
            .get(name)
            .or_else(|| self.parent.as_ref().map(|env| env.cmd_info(name)).flatten())
    }

    fn get_command(
        &self,
        name: &str,
        args: ParsedArgs,
    ) -> Option<Result<Box<dyn Command>, FromArgsError>> {
        self.cmd_info(name).map(|info| (info.from_args)(args))
    }

    pub fn add_binding(&mut self, cmd_info: &dyn CommandInfo) {
        // self.cmds.insert(cmd_info.name(), cmd_info.from_args_fn());
        unimplemented!()
    }

    /// Creates a new environment inheriting from this one.
    fn new_inheriting(self: Rc<Self>) -> Self {
        Self {
            parent: Some(self),
            ..Default::default()
        }
    }
}
