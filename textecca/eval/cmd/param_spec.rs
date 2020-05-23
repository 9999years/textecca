use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParamSpecError {
    #[error("More than one varargs declared")]
    MultipleVarArgs,

    #[error("More than one kwargs declared")]
    MultipleKwArgs,

    #[error("Mandatory param {0} after optional param")]
    MandatoryAfterOptional(String),

    /// A positional param after a keyword-only param (possibly a kwargs).
    #[error("Positional param {0} after keyword-only param")]
    PositionalAfterKw(String),

    /// A positional param after varargs.
    #[error("Positional param {0} after varargs")]
    PositionalAfterVarArgs(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParamSpec(Vec<Param>);

impl ParamSpec {
    pub fn new(params: Vec<Param>) -> Self {
        ParamSpec(params)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Param {
    /// A normal named parameter.
    Normal(NormalParam),

    /// A variable number of arguments, after all mandatory and optional
    /// positional arguments, referred to by the name in the `String`.
    VarArgs(String),

    /// An optional map of keyword arguments, after all mandatory, optional,
    /// varargs, and named keyword-arguments.
    KwArgs(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct NormalParam {
    pub name: String,
    pub required: Required,
    pub keyword: Keyword,
}

impl NormalParam {
    pub fn new(name: String, required: Required, keyword: Keyword) -> Self {
        Self {
            name,
            required,
            keyword,
        }
    }

    /// Creates a new optional positional-only parameter.
    pub fn new_optional_positional(name: String) -> Self {
        Self::new(name, Required::Optional, Keyword::Never)
    }

    /// Creates a new optional parameter.
    pub fn new_optional(name: String) -> Self {
        Self::new(name, Required::Optional, Keyword::Allowed)
    }

    /// Creates a new optional keyword-only parameter.
    pub fn new_optional_keyword(name: String) -> Self {
        Self::new(name, Required::Optional, Keyword::Mandatory)
    }

    /// Creates a new mandatory positional-only parameter.
    pub fn new_positional_only(name: String) -> Self {
        Self::new(name, Required::Mandatory, Keyword::Never)
    }

    /// Creates a new mandatory parameter.
    pub fn new_positional(name: String) -> Self {
        Self::new(name, Required::Mandatory, Keyword::Allowed)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Required {
    Mandatory,
    Optional,
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
