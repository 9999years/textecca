mod html;
mod parse;
pub mod tokenize;

mod ucd_general_category;

mod parse_util;
pub use parse_util::{Error as ParseError, Span};
