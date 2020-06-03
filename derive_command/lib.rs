use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{DeriveInput, Ident, Path};

mod attrs;
mod param;
use attrs::{FieldAttr, StructAttr};
use param::Param;

#[proc_macro_derive(CommandInfo, attributes(textecca))]
pub fn command_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Failed to parse derive input as Rust.");
    impl_command_macro(ast)
}

fn impl_command_macro(ast: syn::DeriveInput) -> TokenStream {
    let DeriveInput {
        attrs,
        ident,
        generics,
        data,
        vis: _vis,
    } = ast;
    let parsed_args_ident: Ident = syn::parse_str("parsed__").unwrap();
    let params = struct_to_params(data);
    let params_code = params.iter().map(|p| p.to_tokens(&parsed_args_ident));

    let struct_attrs = StructAttr::from_attrs(attrs);
    let fields = params.iter().map(|p| &p.field_ident);
    let cmd_name_lit = struct_attrs.cmd_name(&ident);
    let default_parser: Path = syn::parse_str("::textecca::parse::default_parser").unwrap();
    let parser_expr = struct_attrs.parser(&default_parser);

    let gen = quote! {
        impl#generics #ident#generics {
            fn from_args<'a>(
                #parsed_args_ident: &mut ::textecca::cmd::ParsedArgs<'a>,
            ) -> ::std::result::Result<
                    ::std::boxed::Box<dyn ::textecca::cmd::Command<'a> + 'a>,
                    ::textecca::cmd::FromArgsError
            > {
                #(#params_code)*
                #parsed_args_ident.check_no_args()?;
                ::std::result::Result::Ok(::std::boxed::Box::new(#ident {
                    #(#fields,)*
                }))
            }
        }

        impl#generics CommandInfo for #ident#generics {
            fn name() -> String {
                String::from(#cmd_name_lit)
            }

            fn from_args_fn() -> ::textecca::cmd::FromArgs {
                Self::from_args
            }

            fn parser_fn() -> ::textecca::parse::Parser {
                #parser_expr
            }
        }
    };
    gen.into()
}

fn struct_to_params(data: syn::Data) -> Vec<Param> {
    match data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => {
            let mut ret = Vec::with_capacity(named.len());
            for field in named.into_iter().rev() {
                // Named fields always have identifiers.
                let ident = field.ident.unwrap();
                let attrs = FieldAttr::from_attrs(field.attrs);
                ret.push(Param {
                    name: attrs.map(|a| a.name).flatten(),
                    field_ident: ident,
                });
            }
            ret
        }
        _ => panic!("Can only derive textecca::CommandInfo on structs with named fields."),
    }
}
