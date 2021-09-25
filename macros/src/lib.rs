extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Ident, parse_macro_input};

#[proc_macro_attribute]
pub fn game_ref(attr: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as DeriveInput);
    let name = &parsed_input.ident;

    let expanded = quote! {
        #[derive(GameRef, PartialEq, Hash, Eq, Copy, Clone, Debug)]
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
        impl #impl_generics crate::gameref::GameRef for #name #ty_generics #where_clause {
            fn entity(&self) -> Entity {
                self.0
            }
        }
    };

    TokenStream::from(expanded)
}


#[proc_macro_derive(EntityManager)]
pub fn entity_manager_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let ref_name = format_ident!("{}", format!("{}", name).replace("Manager", "Ref"));
    let factor_name = format_ident!("{}", format!("{}", name).replace("Manager", "Factor"));
    // let generics = input.generics;
    // let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl<'a> crate::pops::EntityManager<#ref_name> for #name<'a>  {
            fn get_component<T>(&self, ent_ref: crate::#ref_name) -> &T where T: bevy::ecs::component::Component {
                self.entity_query.get_component::<T>(ent_ref.entity()).unwrap()
            }
            fn get_factor(&self, ent_ref: #ref_name, factor: #factor_name) -> f32 {
                self.get_component::<crate::pops::Factors>(ent_ref).factor(factor)
            }
        }
    };

    TokenStream::from(expanded)
}
