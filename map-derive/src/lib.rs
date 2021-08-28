extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_attribute]
pub fn game_ref(attr: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as DeriveInput);
    let name = &parsed_input.ident;

    let expanded = quote! {
        #[derive(GameRef)]
        #parsed_input
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(GameRef)]
pub fn game_ref_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics crate::pops::GameRef for #name #ty_generics #where_clause {
            fn get(&self, world: &World) -> EntityRef<'_> {
                world.entity(self.0)
            }
        }
    };

    TokenStream::from(expanded)
}
