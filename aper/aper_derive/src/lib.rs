use aper_derive_impl::{impl_state_machine_derive, impl_transition_derive};

extern crate proc_macro;

#[proc_macro_derive(Transition)]
pub fn transition_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    impl_transition_derive(input.into()).into()
}

#[proc_macro_derive(StateMachine)]
pub fn state_machine_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    impl_state_machine_derive(input.into()).into()
}
