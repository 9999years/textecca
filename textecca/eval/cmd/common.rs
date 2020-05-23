use std::io::{self, Write};

use thiserror::Error;

use super::arg_spec::ArgSpec;
use crate::output::doc::{Block, Blocks};

#[derive(Clone, Debug, PartialEq, Error)]
pub enum CallError {
    #[error("Type error: {0}")]
    TypeError(String),
}

pub trait Call {
    fn call(&mut self) -> Result<Blocks, CallError>;
}
