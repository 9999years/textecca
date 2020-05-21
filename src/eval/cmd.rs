use super::arg_spec::ArgSpec;

pub trait Command {
    /// The command's name.
    fn name(&self) -> String;

    /// The arguments this command takes.
    fn arg_spec(&self) -> ArgSpec;
}
