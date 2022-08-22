extern crate proc_macro;

/// Automatic implementation of `StateMachine` for a record struct where every field
/// is also a `StateMachine`.
#[proc_macro_derive(StateMachine)]
pub fn state_machine_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    impl_state_machine_derive(input.into()).into()
}

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{ItemStruct, Type, Visibility};

struct Field<'a> {
    name: &'a Ident,
    ty: &'a Type,
    apply_variant: Ident,
    conflict_variant: Ident,

    transition_ty: TokenStream,
    map_fn_name: Ident,
}

impl<'a> Field<'a> {
    pub fn new(field: &syn::Field) -> Field {
        let name_str =
            inflections::case::to_pascal_case(&field.ident.as_ref().unwrap().to_string());
        let apply_variant = quote::format_ident!("Apply{}", name_str);
        let conflict_variant = quote::format_ident!("{}Conflict", name_str);
        let ty = &field.ty;
        let transition_ty = quote! {
            <#ty as StateMachine>::Transition
        };
        let name = &field.ident.as_ref().unwrap();
        let map_fn_name = quote::format_ident!("map_{}", name.to_string());

        Field {
            name,
            ty: &field.ty,
            apply_variant,
            conflict_variant,
            transition_ty,
            map_fn_name,
        }
    }

    fn generate_accessor(&self, enum_name: &Ident) -> TokenStream {
        let Field {
            name,
            ty,
            map_fn_name,
            apply_variant,
            transition_ty,
            ..
        } = self;

        quote! {
            pub fn #name(&self) -> &#ty {
                &self.#name
            }

            pub fn #map_fn_name(&self, fun: impl FnOnce(&#ty) -> #transition_ty) -> #enum_name {
                #enum_name::#apply_variant(fun(&self.#name))
            }
        }
    }

    fn generate_enum_variant(&self) -> TokenStream {
        let Field {
            apply_variant, ty, ..
        } = self;
        quote! {
            #apply_variant(<#ty as StateMachine>::Transition),
        }
    }

    fn generate_conflict_variant(&self) -> TokenStream {
        let Field {
            conflict_variant,
            ty,
            ..
        } = self;
        quote! {
            #conflict_variant(<#ty as StateMachine>::Conflict),
        }
    }

    fn generate_transition_case(
        &self,
        transition_name: &Ident,
        conflict_name: &Ident,
    ) -> TokenStream {
        let Field {
            name,
            apply_variant,
            conflict_variant,
            ..
        } = self;
        quote! {
            #transition_name::#apply_variant(val) => {
                match self.#name.apply(val) {
                    Ok(v) => {
                        let mut new_self = self.clone();
                        new_self.#name = v;
                        Ok(new_self)
                    },
                    Err(e) => Err(#conflict_name::#conflict_variant(e))
                }
            },
        }
    }
}

fn generate_transform(enum_name: &Ident, fields: &[Field], visibility: &Visibility) -> TokenStream {
    let variants: TokenStream = fields
        .iter()
        .flat_map(Field::generate_enum_variant)
        .collect();

    quote! {
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
        #visibility enum #enum_name {
            #variants
        }
    }
}

fn generate_conflicts(enum_name: &Ident, fields: &[Field], visibility: &Visibility) -> TokenStream {
    let variants: TokenStream = fields
        .iter()
        .flat_map(Field::generate_conflict_variant)
        .collect();

    quote! {
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
        #visibility enum #enum_name {
            #variants
        }
    }
}

fn impl_state_machine_derive(input: TokenStream) -> TokenStream {
    let ast: ItemStruct = syn::parse2(input).expect("Should decorate a struct.");

    let name = &ast.ident;
    let transform_name = quote::format_ident!("{}Transform", name.to_string());
    let conflict_name = quote::format_ident!("{}Conflict", name.to_string());

    let fields: Vec<Field> = match &ast.fields {
        syn::Fields::Named(fields) => fields.named.iter().map(Field::new).collect(),
        _ => panic!("Only structs with named fields can derive StateMachine currently."),
    };

    let accessors: TokenStream = fields
        .iter()
        .flat_map(|e| Field::generate_accessor(e, &transform_name))
        .collect();

    let transition_cases: TokenStream = fields
        .iter()
        .flat_map(|e| Field::generate_transition_case(e, &transform_name, &conflict_name))
        .collect();

    let visibility = &ast.vis;
    let transform_enum = generate_transform(&transform_name, &fields, visibility);
    let conflict_enum = generate_conflicts(&conflict_name, &fields, visibility);

    quote! {
        impl aper::StateMachine for #name {
            type Transition = #transform_name;
            type Conflict = #conflict_name;

            fn apply(&self, transition: &Self::Transition) -> Result<Self, Self::Conflict> {
                match transition {
                    #transition_cases
                }
            }
        }

        impl #name {
            #accessors
        }

        #transform_enum

        #conflict_enum
    }
}
