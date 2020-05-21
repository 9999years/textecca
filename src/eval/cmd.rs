use std::io::{self, Write};

use thiserror::Error;

use super::arg_spec::ArgSpec;
use super::call_args::CallArgs;
use crate::output::doc::{Block, Blocks};

#[derive(Clone, Debug, PartialEq, Error)]
pub enum CallError {
    #[error("Type error: {0}")]
    TypeError(String),
}

pub trait Command {
    /// The command's name.
    fn name() -> String;

    /// The arguments this command takes.
    fn arg_spec() -> ArgSpec;
}

pub trait Call {
    fn call(args: &mut CallArgs) -> Result<Blocks, CallError>;
}
