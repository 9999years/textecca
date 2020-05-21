trait Command {
    fn args(&self) -> ArgSpec;
}

#[derive(Clone, Debug, PartialEq)]
struct ArgSpec(Vec<Arg>);

#[derive(Clone, Debug, PartialEq)]
enum Arg {
    Normal(NormalArg),
    VarArgs(String),
    KwArgs(String),
}

#[derive(Clone, Debug, PartialEq)]
struct NormalArg {
    name: String,
    required: Required,
    keyword: Keyword,
}

#[derive(Clone, Debug, PartialEq)]
enum Required {
    Optional,
    Mandatory,
}

impl Default for Required {
    fn default() -> Self {
        Required::Mandatory
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Keyword {
    /// Positional only
    Never,
    /// Positional or keyword
    Allowed,
    /// Keyword-only
    Mandatory,
}

impl Default for Keyword {
    fn default() -> Self {
        Keyword::Allowed
    }
}

macro_rules! __arg_impl {
    ($name:expr, $req:expr, $kw:expr) => {
        Arg::Normal(NormalArg {
            name: String::from($name),
            required: $req,
            keyword: $kw,
        })
    };
}

macro_rules! arg {
    (opt $name:expr) => {
        __arg_impl!($name, Required::Optional, Keyword::Allowed);
    };
    ($name:expr) => {
        __arg_impl!($name, Required::Mandatory, Keyword::Allowed);
    };
    (opt kw $name:expr) => {
        __arg_impl!($name, Required::Optional, Keyword::Mandatory);
    };
    (kw $name:expr) => {
        __arg_impl!($name, Required::Mandatory, Keyword::Mandatory);
    };
    (opt pos $name:expr) => {
        __arg_impl!($name, Required::Optional, Keyword::Never);
    };
    (pos $name:expr) => {
        __arg_impl!($name, Required::Mandatory, Keyword::Never);
    };
}

// macro_rules! args {
//     ($(arg:tt,)*) => {
//         ArgSpec(vec![
//             $(arg!($arg))*,
//         ])
//     };
// }

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_arg_macro() {
        assert_eq!(
            Arg::Normal(NormalArg {
                name: "argname".to_string(),
                required: Required::Mandatory,
                keyword: Keyword::Allowed,
            }),
            arg!("argname")
        );
    }
}
