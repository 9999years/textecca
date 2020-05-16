// TODO: Remove these.
#![allow(dead_code)]
#![allow(unused_imports)]

mod html;
mod parse;
pub mod tokenize;

#[allow(dead_code)]
mod ucd_general_category;

mod parse_util;
pub use parse_util::{Error as ParseError, Span};

#[cfg(test)]
mod test_util;
