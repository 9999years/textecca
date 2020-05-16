#![allow(unused_imports)]

use std::io::{self, Read};

use nom::{error::VerboseError, IResult};

use textecca::tokenize::{tokenize, Token};
use textecca::Span;

fn span_to_test(s: Span) -> String {
    format!("input.offset({}, {:#?})", s.location_offset(), s.fragment())
}

fn main() -> io::Result<()> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let toks: Result<_, nom::Err<VerboseError<_>>> = tokenize(Span::new(&buf));
    // println!("{:#?}", toks);
    println!("assert_toks!(\n    input,\n    vec![");
    for tok in toks.unwrap().toks {
        println!(
            "        Token::{},",
            match tok {
                Token::Newline(span) => format!("Newline({})", span_to_test(span)),
                Token::Space(span) => format!("Space({})", span_to_test(span)),
                Token::Word(span) => format!("Word({})", span_to_test(span)),
                Token::Punct(span) => format!("Punct({})", span_to_test(span)),
                Token::Num(span) => format!("Num({})", span_to_test(span)),
                Token::Indent(span) => format!("Indent({})", span_to_test(span)),
                t => format!("{:#?}", t),
            }
        );
    }
    println!("    ],\n    {:#?},\n);", buf);
    Ok(())
}
