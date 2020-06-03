use heck::SnakeCase as _;
use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    AttrStyle, Attribute, Expr, ExprLit, ExprPath, Ident, Lit, LitStr, Path,
};

struct MetaNameExpr {
    path: Path,
    #[allow(dead_code)]
    eq_token: syn::token::Eq,
    expr: syn::Expr,
}

impl Parse for MetaNameExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MetaNameExpr {
            path: input.parse()?,
            eq_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

type MetaNameExprList = Punctuated<MetaNameExpr, Comma>;

/// Filters `attrs` to outer attrs and returns their parsed meta.
fn textecca_attrs(attrs: Vec<Attribute>) -> impl Iterator<Item = MetaNameExprList> {
    let textecca_path: Path = syn::parse_str("textecca").unwrap();
    attrs
        .into_iter()
        .filter(move |Attribute { style, path, .. }| {
            *style == AttrStyle::Outer && *path == textecca_path
        })
        .map(|attr| {
            attr.parse_args_with(Punctuated::parse_terminated)
                .expect("Attribute could not be parsed")
        })
}

fn name_expr_attrs(attrs: Vec<Attribute>) -> impl Iterator<Item = MetaNameExpr> {
    textecca_attrs(attrs).flatten()
}

macro_rules! bad_attr_type {
    ($lit:expr) => {
        panic!("Unsupported attribute value type for value {:?}", $lit);
    };
}

fn expr_to_litstr(expr: Expr) -> LitStr {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(lit), ..
    }) = expr
    {
        lit
    } else {
        bad_attr_type!(expr);
    }
}

pub struct FieldAttr {
    pub name: Option<LitStr>,
}

impl FieldAttr {
    fn field_name_expr(expr: Expr) -> LitStr {
        expr_to_litstr(expr)
    }

    // TODO: Unify this with StructAttr init. boilerplate?
    pub fn from_attrs(attrs: Vec<Attribute>) -> Option<Self> {
        let name_path: Path = syn::parse_str("name").unwrap();
        let mut name = None;
        for meta in name_expr_attrs(attrs) {
            if meta.path == name_path {
                name = Some(Self::field_name_expr(meta.expr));
            } else {
                panic!("Unsupported attribute name {:?}", meta.path);
            }
        }
        Some(Self { name })
    }
}

pub struct StructAttr {
    pub name: Option<LitStr>,
    pub parser: Option<Expr>,
}

impl StructAttr {
    fn cmd_name_attr(expr: Expr) -> LitStr {
        expr_to_litstr(expr)
    }

    pub fn from_attrs(attrs: Vec<Attribute>) -> Self {
        let name_path: Path = syn::parse_str("name").unwrap();
        let parser_path: Path = syn::parse_str("parser").unwrap();

        let mut name = None;
        let mut parser = None;
        for meta in name_expr_attrs(attrs) {
            if meta.path == name_path {
                name = Some(Self::cmd_name_attr(meta.expr));
            } else if meta.path == parser_path {
                parser = Some(meta.expr);
            } else {
                panic!("Unsupported attribute name {:?}", meta.path);
            }
        }
        Self { name, parser }
    }

    pub fn cmd_name(&self, default: &Ident) -> LitStr {
        self.name.clone().unwrap_or_else(|| {
            LitStr::new(&(default.to_string()).to_snake_case(), Span::call_site())
        })
    }

    pub fn parser(&self, default: &Path) -> Expr {
        self.parser.clone().unwrap_or_else(|| {
            Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: default.clone(),
            })
        })
    }
}
