use derive_more::{From, Into};
use quote::ToTokens;
use syn::{parse::Parse, Attribute};

#[derive(From, Into)]
pub struct Attrs {
    pub attrs: Vec<syn::Attribute>,
}

impl Parse for Attrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        Ok(Attrs { attrs })
    }
}

impl ToTokens for Attrs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
    }
}
