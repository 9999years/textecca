use std::io::{self, Read};

use nom::{error::VerboseError, IResult};

use textecca::parse;

fn main() -> io::Result<()> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let tree: IResult<_, _, VerboseError<_>> = parse::parse(&buf);
    println!("{:#?}", tree);
    Ok(())
}
