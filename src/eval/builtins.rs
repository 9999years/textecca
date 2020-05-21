#[macro_use]
use super::arg_spec::ArgSpec;
use super::cmd::Command;

pub struct Section;

impl Command for Section {
    fn name(&self) -> String {
        "sec".to_string()
    }

    fn arg_spec(&self) -> ArgSpec {
        ArgSpec::new(vec![
            arg!(var "args"),
            arg!(kw "kwargs"),
            arg!(opt pos "optional positional"),
        ])
    }
}
