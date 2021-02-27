extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(StateMachine)]
pub fn state_machine_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_state_machine_derive(&ast)
}

fn impl_state_machine_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl aper::StateMachine for #name {
            type Transition = ();

            fn apply(&mut self, transition: Self::Transition) {

            }
        }
    };
    gen.into()
}
