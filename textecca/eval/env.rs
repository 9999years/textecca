use std::collections::HashMap;
use std::rc::Rc;

use super::cmd::{Command, CommandInfo, FromArgsError, FromArgsFn, ParsedRawArgs};

pub struct Environment {
    parent: Option<Rc<Environment>>,
    cmds: HashMap<String, FromArgsFn>,
}

impl Environment {
    pub fn is_bound(&self, name: &str) -> bool {
        self.cmds.contains_key(name)
            || self
                .parent
                .as_ref()
                .map(|env| env.is_bound(name))
                .unwrap_or(false)
    }

    fn from_args_fn(&self, name: &str) -> Option<&FromArgsFn> {
        self.cmds.get(name).or_else(|| {
            self.parent
                .as_ref()
                .map(|env| env.from_args_fn(name))
                .flatten()
        })
    }

    fn get_command(
        &self,
        name: &str,
        args: ParsedRawArgs,
    ) -> Option<Result<Box<dyn Command>, FromArgsError>> {
        self.from_args_fn(name).map(|f| f(args))
    }

    pub fn add_binding(&mut self, cmd_info: &dyn CommandInfo) {
        self.cmds.insert(cmd_info.name(), cmd_info.from_args_fn());
    }
}
