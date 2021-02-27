extern crate proc_macro;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn;
use syn::{ItemStruct, Type};

#[proc_macro_derive(StateMachine)]
pub fn state_machine_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: ItemStruct = syn::parse(input.into()).expect("Should decorate a struct.");

    impl_state_machine_derive(&ast).into()
}

struct Field<'a> {
    name: &'a Ident,
    ty: &'a Type,
    apply_variant: Ident,
    transition_ty: TokenStream,
    map_fn_name: Ident,
}

impl<'a> Field<'a> {
    pub fn new(field: &syn::Field) -> Field {
        let name_str =
            inflections::case::to_pascal_case(&field.ident.as_ref().unwrap().to_string());
        let apply_variant = quote::format_ident!("Apply{}", name_str);
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
            transition_ty,
            map_fn_name
        }
    }

    fn generate_accessor(&self, enum_name: &Ident) -> TokenStream {
        let Field { name, ty, map_fn_name, apply_variant , transition_ty} = self;

        let gen = quote! {
            pub fn #name(&self) -> &#ty {
                &self.#name
            }

            pub fn #map_fn_name(&self, fun: impl FnOnce(&#ty) -> #transition_ty) -> #enum_name {
                #enum_name::#apply_variant(fun(&self.#name))
            }
        };
        gen.into()
    }

    fn generate_enum_variant(&self) -> TokenStream {
        let Field { apply_variant, ty, .. } = self;
        let gen: TokenStream = quote! {
            #apply_variant(<#ty as StateMachine>::Transition),
        };
        gen.into()
    }

    fn generate_transition_case(&self, enum_name: &Ident) -> TokenStream {
        let Field {
            name,
            apply_variant,
            ..
        } = self;
        let gen: TokenStream = quote! {
            #enum_name::#apply_variant(val) => self.#name.apply(val),
        };
        gen.into()
    }
}

fn generate_transform(enum_name: &Ident, fields: &Vec<Field>) -> TokenStream {
    let variants: TokenStream = fields
        .iter()
        .flat_map(Field::generate_enum_variant)
        .collect();

    let gen = quote! {
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
        enum #enum_name {
            #variants
        }
    };

    gen.into()
}

fn impl_state_machine_derive(ast: &ItemStruct) -> TokenStream {
    let name = &ast.ident;
    let enum_name = quote::format_ident!("{}Transform", name.to_string());

    let fields: Vec<Field> = match &ast.fields {
        syn::Fields::Named(fields) => fields.named.iter().map(Field::new).collect(),
        _ => panic!("Only structs with named fields can derive StateMachine currently."),
    };

    let accessors: TokenStream = fields
        .iter()
        .flat_map(|e| Field::generate_accessor(e, &enum_name))
        .collect();

    let transition_cases: TokenStream = fields
        .iter()
        .flat_map(|e| Field::generate_transition_case(e, &enum_name))
        .collect();

    let transform_enum = generate_transform(&enum_name, &fields);

    let gen = quote! {
        impl aper::StateMachine for #name {
            type Transition = #enum_name;

            fn apply(&mut self, transition: Self::Transition) {
                match transition {
                    #transition_cases
                };
            }
        }

        impl #name {
            #accessors
        }

        #transform_enum
    };
    gen.into()
}
