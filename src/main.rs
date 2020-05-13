use std::io::{self, Read};

use textecca::parse;

fn main() -> io::Result<()> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let tree = parse::parse(&buf).unwrap();
    println!("{:#?}", tree);
    Ok(())
}
