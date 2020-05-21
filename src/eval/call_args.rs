use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct CallArgs {
    pub args: HashMap<String, String>,
    pub varargs: Vec<String>,
    pub kwargs: HashMap<String, String>,
}
