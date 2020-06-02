use heck::SnakeCase as _;
use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, token::Comma, AttrStyle, Attribute, Ident, Lit, LitStr, Meta, MetaList,
    MetaNameValue, NestedMeta, Path,
};

/// Filters `attrs` to outer attrs and returns their parsed meta.
fn textecca_attrs(attrs: Vec<Attribute>) -> impl Iterator<Item = Punctuated<NestedMeta, Comma>> {
    let textecca_path: Path = syn::parse_str("textecca").unwrap();
    attrs
        .into_iter()
        .filter(|Attribute { style, .. }| *style == AttrStyle::Outer)
        .map(move |attr| {
            attr.parse_meta()
                .map(|meta| match meta {
                    Meta::List(MetaList { path, nested, .. }) if path == textecca_path => {
                        Some(nested)
                    }
                    _ => None,
                })
                .expect("Attribute could not be parsed")
        })
        .flatten()
}

fn nameval_attrs(attrs: Vec<Attribute>) -> impl Iterator<Item = MetaNameValue> {
    textecca_attrs(attrs).flatten().map(|meta| match meta {
        NestedMeta::Meta(Meta::NameValue(meta)) => meta,
        _ => panic!("Unsupported attribute {:?}", meta),
    })
}

pub struct FieldAttr {
    pub name: Option<LitStr>,
}

impl FieldAttr {
    // TODO: Unify this with StructAttr init. boilerplate?
    pub fn from_attrs(attrs: Vec<Attribute>) -> Option<Self> {
        let name_path: Path = syn::parse_str("name").unwrap();
        let mut name = None;
        for meta in nameval_attrs(attrs) {
            if meta.path != name_path {
                panic!("Unsupported attribute name {:?}", meta.path);
            }
            match meta.lit {
                Lit::Str(lit) => {
                    name = Some(lit);
                }
                lit => {
                    panic!("Unsupported attribute value type for value {:?}", lit);
                }
            }
        }
        Some(Self { name })
    }
}

pub struct StructAttr {
    pub name: Option<LitStr>,
}

impl StructAttr {
    pub fn from_attrs(attrs: Vec<Attribute>) -> Self {
        let name_path: Path = syn::parse_str("name").unwrap();
        let mut name = None;
        for meta in nameval_attrs(attrs) {
            if meta.path != name_path {
                panic!("Unsupported attribute name {:?}", meta.path);
            }
            match meta.lit {
                Lit::Str(lit) => {
                    name = Some(lit);
                }
                lit => {
                    panic!("Unsupported attribute value type for value {:?}", lit);
                }
            }
        }
        Self { name }
    }

    pub fn cmd_name(&self, default: &Ident) -> LitStr {
        self.name.clone().unwrap_or_else(|| {
            LitStr::new(&(default.to_string()).to_snake_case(), Span::call_site())
        })
    }
}
