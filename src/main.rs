#![allow(unused_imports)]

use std::io::{self, Read};

use nom::{error::VerboseError, IResult};

use textecca::tokenize::tokenize;
use textecca::Span;

fn main() -> io::Result<()> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let toks: Result<_, nom::Err<VerboseError<_>>> = tokenize(Span::new(&buf));
    println!("{:#?}", toks);
    Ok(())
}
