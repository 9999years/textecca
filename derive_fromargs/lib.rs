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

fn try_from_impl(cmd_name: &str, data: syn::Data) {
    match data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => {
            // TODO: Which attribute names indicate "previous parameters are
            // positional-only" and "following parameters are keyword-only"?
            // #[end_pos_only]
            // #[begin_kw_only]
            unimplemented!()
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
