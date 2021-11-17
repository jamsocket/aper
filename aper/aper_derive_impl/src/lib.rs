use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Item, ItemStruct, Type, Visibility};

pub fn impl_transition_derive(input: proc_macro2::TokenStream) -> TokenStream {
    let ast: Item = syn::parse2(input).expect("Should decorate a struct.");

    let name = match ast {
        Item::Enum(it) => it.ident,
        Item::Struct(it) => it.ident,
        _ => panic!("Can only derive Transition for an enum or struct."),
    };

    quote! {
        impl aper::Transition for #name {}
    }
}

pub struct Field<'a> {
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

    fn generate_transition_case(&self, enum_name: &Ident) -> TokenStream {
        let Field {
            name,
            apply_variant,
            ..
        } = self;
        quote! {
            #enum_name::#apply_variant(val) => self.#name.apply(val),
        }
    }
}

pub fn generate_transform(enum_name: &Ident, fields: &[Field], visibility: &Visibility) -> TokenStream {
    let variants: TokenStream = fields
        .iter()
        .flat_map(Field::generate_enum_variant)
        .collect();

    quote! {
        #[derive(aper::Transition, serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
        #visibility enum #enum_name {
            #variants
        }
    }
}

pub fn impl_state_machine_derive(input: TokenStream) -> TokenStream {
    let ast: ItemStruct = syn::parse2(input).expect("Should decorate a struct.");

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

    let visibility = &ast.vis;
    let transform_enum = generate_transform(&enum_name, &fields, visibility);

    quote! {
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
    }
}
