use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, LitStr};

pub struct Param {
    pub name: Option<LitStr>,
    pub field_ident: Ident,
}

impl Param {
    pub fn name(&self) -> LitStr {
        self.name.clone().unwrap_or_else(|| {
            LitStr::new(
                &(self.field_ident.to_string()).replace("_", " "),
                Span::call_site(),
            )
        })
    }

    pub fn to_tokens(&self, parsed_args_ident: &Ident) -> TokenStream {
        let Self { field_ident, .. } = self;
        let name = self.name();
        quote! {
            let #field_ident = #parsed_args_ident.pop_mandatory(#name)?;
        }
    }
}
