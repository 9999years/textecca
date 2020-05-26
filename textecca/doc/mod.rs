//! Document structure types.
//!
//! These types are used to represent a *rendered* document. Textecca's markup
//! language parses and renders into `Block`s, and then serializers (see the
//! `ser` module) render `Block`s into a particular output format.
mod blocks;
mod builder;
mod inlines;
mod length;
mod structure;

pub use blocks::*;
pub use builder::*;
pub use inlines::*;
pub use length::*;
pub use structure::*;
