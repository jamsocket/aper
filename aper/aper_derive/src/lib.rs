use aper_derive_impl::impl_state_machine_derive;

extern crate proc_macro;

/// Automatic implementation of `StateMachine` for a record struct where every field
/// is also a `StateMachine`.
#[proc_macro_derive(StateMachine)]
pub fn state_machine_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    impl_state_machine_derive(input.into()).into()
}
