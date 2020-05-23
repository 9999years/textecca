use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ArgSpecError {
    #[error("More than one varargs declared")]
    MultipleVarArgs,

    #[error("More than one kwargs declared")]
    MultipleKwArgs,

    #[error("Mandatory arg {0} after optional arg")]
    MandatoryAfterOptional(String),

    /// A positional argument after a keyword-only argument, including a kwargs.
    #[error("Positional arg {0} after keyword-only arg")]
    PositionalAfterKw(String),

    /// A positional argument after varargs is given.
    #[error("Positional arg {0} after varargs")]
    PositionalAfterVarArgs(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArgSpec(Vec<Arg>);

impl ArgSpec {
    pub fn new(args: Vec<Arg>) -> Self {
        ArgSpec(args)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Arg {
    /// A normal named argument.
    Normal(NormalArg),

    /// A variable number of arguments, after all mandatory and optional
    /// positional arguments, referred to by the name in the `String`.
    VarArgs(String),

    /// An optional map of keyword arguments, after all mandatory, optional,
    /// varargs, and named keyword-arguments.
    KwArgs(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct NormalArg {
    pub name: String,
    pub required: Required,
    pub keyword: Keyword,
}

impl NormalArg {
    pub fn new(name: String, required: Required, keyword: Keyword) -> Self {
        Self {
            name,
            required,
            keyword,
        }
    }

    /// Creates a new optional positional-only argument.
    pub fn new_optional_positional(name: String) -> Self {
        Self::new(name, Required::Optional, Keyword::Never)
    }

    /// Creates a new optional argument.
    pub fn new_optional(name: String) -> Self {
        Self::new(name, Required::Optional, Keyword::Allowed)
    }

    /// Creates a new optional keyword-only argument.
    pub fn new_optional_keyword(name: String) -> Self {
        Self::new(name, Required::Optional, Keyword::Mandatory)
    }

    /// Creates a new mandatory positional-only argument.
    pub fn new_positional_only(name: String) -> Self {
        Self::new(name, Required::Mandatory, Keyword::Never)
    }

    /// Creates a new mandatory argument.
    pub fn new_positional(name: String) -> Self {
        Self::new(name, Required::Mandatory, Keyword::Allowed)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Required {
    Optional,
    Mandatory,
}

impl Default for Required {
    fn default() -> Self {
        Required::Mandatory
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
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

enum ArgKind {
    Normal,
    VarArgs,
    KwArgs,
}

macro_rules! arg {
    (var $name:expr) => {
        $crate::eval::cmd::arg_spec::Arg::VarArgs(String::from($name))
    };
    (kw $name:expr) => {
        $crate::eval::cmd::arg_spec::Arg::KwArgs(String::from($name))
    };
    (opt pos $name:expr) => {
        $crate::eval::cmd::arg_spec::Arg::Normal(
            $crate::eval::cmd::arg_spec::NormalArg::new_optional_positional(String::from($name)),
        )
    };
    (opt $name:expr) => {
        $crate::eval::cmd::arg_spec::Arg::Normal(
            $crate::eval::cmd::arg_spec::NormalArg::new_optional(String::from($name)),
        )
    };
    (opt kw $name:expr) => {
        $crate::eval::cmd::arg_spec::Arg::Normal(
            $crate::eval::cmd::arg_spec::NormalArg::new_optional_keyword(String::from($name)),
        )
    };
    (pos $name:expr) => {
        $crate::eval::cmd::arg_spec::Arg::Normal(
            $crate::eval::cmd::arg_spec::NormalArg::new_positional_only(String::from($name)),
        )
    };
    ($name:expr) => {
        $crate::eval::cmd::arg_spec::Arg::Normal(
            $crate::eval::cmd::arg_spec::NormalArg::new_positional(String::from($name)),
        )
    };
}
