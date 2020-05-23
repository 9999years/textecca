// #[macro_use]
// use super::arg_spec::ArgSpec;
// use super::call_args::CallArgs;
// use super::cmd::{Call, CallError, Command};
// use crate::output::doc::{Block, Blocks, Heading, Inline};

// macro_rules! impl_cmd {
//     ($ty:ident, name = $name:expr, args = $($args:tt)*) => {
//         impl Command for $ty {
//             fn name() -> String {
//                 ($name).into()
//             }

//             fn arg_spec() -> ArgSpec {
//                 ArgSpec::new(vec![$($args)*])
//             }
//         }
//     };
// }

// pub struct Section;

// impl_cmd!(Section, name = "sec", args = arg!("name"));
// impl Call for Section {
//     fn call(args: &mut CallArgs) -> Result<Blocks, CallError> {
//         let name = args.args.remove("name").unwrap();
//         Ok(vec![Block::Heading(Heading {
//             level: 1,
//             text: vec![Inline::Text(name).into()],
//         })
//         .into()])
//     }
// }
