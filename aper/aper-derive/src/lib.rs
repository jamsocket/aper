use proc_macro2::Ident;
use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeSet;

#[proc_macro_derive(AperSync)]
pub fn attach_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let state = MacroState::from_tokens(input.into());
    let result = state.generate_impl();
    result.into()
}

enum StructType {
    Record(BTreeSet<String>),
    Tuple(usize),
    Unit,
}

struct MacroState {
    name: Ident,
    fields: StructType,
}

impl MacroState {
    fn from_tokens(tokens: TokenStream) -> Self {
        let ast = syn::parse2::<syn::ItemStruct>(tokens.clone()).unwrap();
        let name = ast.ident;
        let fields = match ast.fields {
            syn::Fields::Named(fields) => {
                let fields = fields
                    .named
                    .iter()
                    .map(|field| {
                        let name = field.ident.as_ref().unwrap().to_string();
                        name
                    })
                    .collect();
                StructType::Record(fields)
            }
            syn::Fields::Unnamed(fields) => {
                let fields = fields.unnamed.len();
                StructType::Tuple(fields)
            }
            syn::Fields::Unit => StructType::Unit,
        };
        Self { name, fields }
    }

    fn generate_impl(&self) -> TokenStream {
        let name = &self.name;
        let fields = match &self.fields {
            StructType::Record(fields) => {
                let fields = fields.iter().map(|field| {
                    let field = syn::Ident::new(field, proc_macro2::Span::call_site());
                    let name = Literal::byte_string(field.to_string().as_bytes());
                    quote! {
                        #field: aper::AperSync::attach(store.child(#name))
                    }
                });
                quote! {
                    #name {
                        #(#fields),*
                    }
                }
            }
            StructType::Tuple(fields) => {
                let fields = (0..*fields).map(|i| {
                    let i = Literal::byte_string(i.to_be_bytes().as_slice());
                    quote! {
                        aper::AperSync::attach(store.child(#i))
                    }
                });
                quote! {
                    #name(#(#fields),*)
                }
            }
            StructType::Unit => quote! {
                #name
            },
        };

        quote! {
            impl aper::AperSync for #name {
                fn attach(mut store: aper::StoreHandle) -> Self {
                    #fields
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_impl_for_empty_struct() {
        let input = quote! {
            struct MyStruct;
        };

        let state = MacroState::from_tokens(input);
        let result = state.generate_impl();

        let expected = quote! {
            impl aper::AperSync for MyStruct {
                fn attach(mut store: aper::StoreHandle) -> Self {
                    MyStruct
                }
            }
        };

        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_impl_for_struct_with_named_fields() {
        let input = quote! {
            struct MyStruct {
                field1: i32,
                field2: String,
            }
        };

        let state = MacroState::from_tokens(input);
        let result = state.generate_impl();

        let expected = quote! {
            impl aper::AperSync for MyStruct {
                fn attach(mut store: aper::StoreHandle) -> Self {
                    MyStruct {
                        field1: aper::AperSync::attach(store.child(b"field1")),
                        field2: aper::AperSync::attach(store.child(b"field2"))
                    }
                }
            }
        };

        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_impl_for_struct_with_tuple_fields() {
        let input = quote! {
            struct MyStruct(i32, String);
        };

        let state = MacroState::from_tokens(input);
        let result = state.generate_impl();

        let expected = quote! {
            impl aper::AperSync for MyStruct {
                fn attach(mut store: aper::StoreHandle) -> Self {
                    MyStruct(
                        aper::AperSync::attach(store.child(b"\0\0\0\0\0\0\0\0")),
                        aper::AperSync::attach(store.child(b"\0\0\0\0\0\0\0\x01"))
                    )
                }
            }
        };

        assert_eq!(result.to_string(), expected.to_string());
    }
}
