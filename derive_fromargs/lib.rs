use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use proc_macro::{self, TokenStream};
use proc_macro2::Span;
use quote::quote;
use syn;

use inflector::Inflector;

#[proc_macro_derive(Command)]
pub fn command_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Failed to parse derive input as Rust.");
    impl_command_macro(&ast)
}

// pub trait Command: TryFrom<ParsedRawArgs, Error = FromArgsError> {
//     /// The command's name.
//     fn name() -> String;
// }

// #[textecca(name = "xxx")]

fn get_attrs(
    attrs: &[syn::Attribute],
    expect_path: syn::Path,
) -> impl Iterator<Item = syn::Meta> + '_ {
    attrs
        .iter()
        .filter(|syn::Attribute { style, .. }| *style == syn::AttrStyle::Outer)
        .map(move |attr| {
            attr.parse_meta()
                .map(|meta| match &meta {
                    syn::Meta::NameValue(syn::MetaNameValue { path, .. })
                    | syn::Meta::Path(path)
                    | syn::Meta::List(syn::MetaList { path, .. }) => {
                        if *path == expect_path {
                            Some(meta)
                        } else {
                            None
                        }
                    }
                })
                .unwrap_or_default()
        })
        .flatten()
}

fn textecca_attr(attrs: &[syn::Attribute], expect_path: syn::Path) -> Option<syn::Lit> {
    get_attrs(attrs, expect_path)
        .map(|meta| {
            if let syn::Meta::NameValue(syn::MetaNameValue { lit, .. }) = meta {
                Some(lit)
            } else {
                None
            }
        })
        .flatten()
        .next()
}

macro_rules! syn_path {
    ($($component:literal),*) => {
        syn::Path {
            leading_colon: None,
            segments: {
                let mut path = syn::punctuated::Punctuated::new();
                $(
                    path.push(syn::PathSegment {
                        ident: syn::Ident::new($component, Span::call_site()),
                        arguments: syn::PathArguments::None,
                    });
                )*
                path
            },
        }
    };
}

fn textecca_name(attrs: &[syn::Attribute], default: &syn::Ident) -> String {
    textecca_attr(attrs, syn_path!("textecca", "name"))
        .map(|lit| match lit {
            syn::Lit::Str(name) => name.value(),
            _ => panic!("textecca::name attribute must be a string literal."),
        })
        .unwrap_or_else(|| default.to_string().to_snake_case())
}

#[derive(Clone)]
enum ParamRequired {
    Mandatory,
    /// Optional with default value
    Optional(Option<String>),
}

#[derive(Clone, Copy)]
enum KeywordRequired {
    Never,
    Allowed,
    Mandatory,
}

#[derive(Clone, Copy)]
enum ParamKind {
    Normal,
    VarArgs,
    KwArgs,
}

struct Param {
    ident: syn::Ident,
    /// Parameter name, if different from the identifier name.
    name: Option<String>,
    required: ParamRequired,
    keyword: KeywordRequired,
    kind: ParamKind,
}

impl Param {
    fn get_type(&self) -> syn::Type {
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: match &self.kind {
                ParamKind::Normal => match &self.required {
                    ParamRequired::Mandatory => syn::parse_str("::std::string::String").unwrap(),
                    ParamRequired::Optional(_) => {
                        syn::parse_str("::std::option::Option<::std::string::String>").unwrap()
                    }
                },
                ParamKind::VarArgs => {
                    syn::parse_str("::std::vec::Vec<::std::string::String>").unwrap()
                }
                ParamKind::KwArgs => syn::parse_str(
                    "::std::collections::HashMap<::std::string::String, ::std::string::String>",
                )
                .unwrap(),
            },
        })
    }

    fn name(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.ident.to_string())
    }

    fn from_args(&self) -> proc_macro2::TokenStream {
        // TODO: figure out how to turn Vec<Param> into code to fetch the params.
        let name_str = syn::Lit::Str(syn::LitStr::new(&self.name(), Span::call_site()));
        quote! {}
    }
}

fn type_matches(
    ty: &syn::Type,
    last_segment: &str,
    generic_args_match: impl Fn(
        &syn::punctuated::Punctuated<syn::GenericArgument, syn::token::Comma>,
    ) -> bool,
) -> bool {
    match ty {
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path { segments, .. },
        }) => {
            let syn::PathSegment { ident, arguments } =
                segments.last().expect("Type should be non-empty.");
            ident == last_segment
                && match arguments {
                    syn::PathArguments::AngleBracketed(bracketed) => {
                        generic_args_match(&bracketed.args)
                    }
                    _ => false,
                }
        }
        _ => false,
    }
}

fn is_string(ty: &syn::Type) -> bool {
    type_matches(ty, "String", |_| false)
}

fn is_option_string(ty: &syn::Type) -> bool {
    type_matches(ty, "Option", |args| {
        args.len() == 1
            && match args.first().unwrap() {
                syn::GenericArgument::Type(ty) => is_string(ty),
                _ => false,
            }
    })
}

fn is_vec_string(ty: &syn::Type) -> bool {
    type_matches(ty, "Vec", |args| {
        args.len() == 1
            && match args.first().unwrap() {
                syn::GenericArgument::Type(ty) => is_string(ty),
                _ => false,
            }
    })
}

fn is_hashmap_string_string(ty: &syn::Type) -> bool {
    type_matches(ty, "HashMap", |args| {
        args.len() == 1
            && match args.first().unwrap() {
                syn::GenericArgument::Type(ty) => is_string(ty),
                _ => false,
            }
            && match &args[1] {
                syn::GenericArgument::Type(ty) => is_string(&ty),
                _ => false,
            }
    })
}

fn struct_to_params(data: syn::Data) -> Vec<Param> {
    match data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => {
            // TODO: Which attribute names indicate "previous parameters are
            // positional-only" and "following parameters are keyword-only"?
            let prev_are_pos = syn_path!("end_pos_only");
            let following_are_kw = syn_path!("begin_kw_only");
            let name_path = syn_path!("name");
            let default_path = syn_path!("default");

            let mut seen_prev_are_pos = false;
            let mut seen_following_are_kw = false;
            // let mut seen_kwargs = false;
            // let mut seen_varargs = false;
            let mut kw_required = KeywordRequired::Allowed;
            let mut required = ParamRequired::Mandatory;
            let mut ret = Vec::<Param>::with_capacity(named.len());
            for field in named {
                let mut name = None;
                let mut kind = ParamKind::Normal;
                for meta in get_attrs(&field.attrs, prev_are_pos.clone()) {
                    if let syn::Meta::Path(path) = meta {
                        if path == prev_are_pos {
                            if seen_prev_are_pos {
                                panic!(
                                    "Only one #[{:?}] attribute is allowed per command.",
                                    prev_are_pos
                                );
                            }
                            seen_prev_are_pos = true;
                            let iter = ret.iter_mut();
                            for param in iter {
                                param.keyword = KeywordRequired::Never;
                            }
                        } else if path == following_are_kw {
                            if seen_following_are_kw {
                                panic!(
                                    "Only one #[{:?}] attribute is allowed per command.",
                                    following_are_kw
                                );
                            }
                            seen_following_are_kw = true;
                            kw_required = KeywordRequired::Mandatory;
                        }
                    } else if let syn::Meta::NameValue(meta) = meta {
                        if meta.path == name_path {
                            name = match meta.lit {
                                syn::Lit::Str(lit) => Some(lit.value()),
                                _ => panic!(
                                    "Parameter name for {} is not a string.",
                                    field.ident.unwrap(),
                                ),
                            };
                        } else if meta.path == default_path {
                            required = ParamRequired::Optional(match meta.lit {
                                syn::Lit::Str(lit) => Some(lit.value()),
                                _ => panic!(
                                    "Default parameter value for {} is not a string.",
                                    field.ident.unwrap(),
                                ),
                            });
                        }
                    }
                }

                let required = match &required {
                    ParamRequired::Mandatory => {
                        if is_option_string(&field.ty) {
                            ParamRequired::Optional(None)
                        } else {
                            ParamRequired::Mandatory
                        }
                    }
                    _ => required.clone(),
                };

                if is_vec_string(&field.ty) {
                    kind = ParamKind::VarArgs;
                } else if is_hashmap_string_string(&field.ty) {
                    kind = ParamKind::KwArgs;
                }

                ret.push(Param {
                    ident: field.ident.unwrap(),
                    name,
                    required: required.clone(),
                    keyword: kw_required,
                    kind,
                })
            }
            ret
        }
        _ => panic!("Can only derive textecca::Command on structs with named fields."),
    }
}

fn impl_command_macro(ast: &syn::DeriveInput) -> TokenStream {
    dbg!(ast);
    let name = &ast.ident;
    let cmd_name = textecca_name(&ast.attrs, name);
    let cmd_name_lit = syn::LitStr::new(&cmd_name, Span::call_site());
    let gen = quote! {
        impl TryFrom<ParsedRawArgs> for #name {
            type Error = FromArgsError;

            fn try_from(args: ParsedRawArgs) -> Result<Self, Self::Error> {
                unimplemented!()
            }
        }

        impl Command for #name {
            fn name() -> String {
                String::from(#cmd_name_lit)
            }
        }
    };
    gen.into()
}
