#![feature(log_syntax)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, PathArguments, parse_macro_input};

// Simplified and easily queriable type struct
// Only maps types that don't have any form of arguments,
// and angle bracketed types that have only one argument
struct Type {
    pub ty: syn::Ident,
    pub sub_ty: Option<Box<Type>>,
}

impl TryFrom<&syn::Type> for Type {
    type Error = ();

    fn try_from(value: &syn::Type) -> Result<Self, Self::Error> {
        let path = match value {
            syn::Type::Path(type_path) => type_path,
            _ => return Err(()),
        };

        if path.path.segments.len() != 1 {
            return Err(());
        }

        let segment = path.path.segments.first().unwrap();

        let sub_ty = match &segment.arguments {
            PathArguments::Parenthesized(_) => return Err(()),
            PathArguments::AngleBracketed(bracketed) => {
                let args = &bracketed.args;

                if args.len() != 1 {
                    return Err(());
                }

                let ty = match args.first().unwrap() {
                    syn::GenericArgument::Type(ty) => ty,
                    _ => return Err(()),
                };

                Some(Box::new(ty.try_into()?))
            }
            PathArguments::None => None,
        };

        Ok(Self {
            ty: segment.ident.clone(),
            sub_ty,
        })
    }
}

fn gen_builder_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_ident = match &field.ident {
        Some(ident) => ident,
        None => panic!("Fields must have names"),
    };

    let base_type: Type = match (&field.ty).try_into() {
        Ok(ty) => ty,
        Err(_) => {
            panic!("Invalid type. Only supported types are: String, f64, Vec<u8> and DateTime<Utc>")
        }
    };

    match base_type.ty.to_string().as_str() {
        "String" => {
            quote! {
                .with_string_field(stringify!(#field_ident), |instance| Some(instance.#field_ident.clone()))
            }
        }
        "f64" => {
            quote! {
                .with_number_field(stringify!(#field_ident), |instance| Some(instance.#field_ident))
            }
        }
        "bool" => {
            quote! {
                .with_boolean_field(stringify!(#field_ident), |instance| Some(instance.#field_ident))
            }
        }
        "DateTime" => {
            if base_type.sub_ty.as_ref().is_some_and(|sub| sub.ty == "Utc") {
                quote! {
                    .with_datetime_field(stringify!(#field_ident), |instance| Some(instance.#field_ident))
                }
            } else {
                panic!("DateTime fields have to be DateTime<Utc>");
            }
        }
        "Vec" => {
            let vec_type = base_type.sub_ty.as_ref().unwrap();

            match vec_type.ty.to_string().as_str() {
                "u8" => {
                    quote! {
                        .with_raw_field(stringify!(#field_ident), |instance| Some(instance.#field_ident.clone()))
                    }
                }
                "String" => {
                    quote! {
                        .with_string_list_field(stringify!(#field_ident), |instance| Some(instance.#field_ident.clone()))
                    }
                }
                "f64" => {
                    quote! {
                        .with_number_list_field(stringify!(#field_ident), |instance| Some(instance.#field_ident.clone()))
                    }
                }
                "bool" => {
                    quote! {
                        .with_boolean_list_field(stringify!(#field_ident), |instance| Some(instance.#field_ident.clone()))
                    }
                }
                "DateTime" => {
                    if vec_type.sub_ty.as_ref().is_some_and(|sub| sub.ty == "Utc") {
                        quote! {
                            .with_datetime_list_field(stringify!(#field_ident), |instance| Some(instance.#field_ident.clone()))
                        }
                    } else {
                        panic!("DateTime fields have to be DateTime<Utc>");
                    }
                }
                "Vec" => {
                    let vec_type = vec_type.sub_ty.as_ref().unwrap();

                    match vec_type.ty.to_string().as_str() {
                        "u8" => {
                            quote! {
                                .with_raw_list_field(stringify!(#field_ident), |instance| Some(instance.#field_ident.clone()))
                            }
                        }
                        _ => {
                            panic!(
                                "Invalid type. Only supported types are: String, f64, Vec<u8> and DateTime<Utc>"
                            );
                        }
                    }
                }
                _ => {
                    panic!(
                        "Invalid type. Only supported types are: String, f64, Vec<u8> and DateTime<Utc>"
                    );
                }
            }
        }
        "Option" => {
            let option_type = base_type.sub_ty.as_ref().unwrap();

            match option_type.ty.to_string().as_str() {
                "String" => {
                    quote! {
                        .with_string_field(stringify!(#field_ident), |instance| instance.#field_ident.clone())
                    }
                }
                "f64" => {
                    quote! {
                        .with_number_field(stringify!(#field_ident), |instance| instance.#field_ident)
                    }
                }
                "bool" => {
                    quote! {
                        .with_boolean_field(stringify!(#field_ident), |instance| instance.#field_ident)
                    }
                }
                "DateTime" => {
                    if base_type.sub_ty.as_ref().is_some_and(|sub| sub.ty == "Utc") {
                        quote! {
                            .with_datetime_field(stringify!(#field_ident), |instance| instance.#field_ident)
                        }
                    } else {
                        panic!("DateTime fields have to be DateTime<Utc>");
                    }
                }
                "Vec" => {
                    let vec_type = &option_type.sub_ty.as_ref().unwrap();

                    match vec_type.ty.to_string().as_str() {
                        "u8" => {
                            quote! {
                                .with_raw_field(stringify!(#field_ident), |instance| instance.#field_ident.clone())
                            }
                        }
                        "String" => {
                            quote! {
                                .with_string_list_field(stringify!(#field_ident), |instance| instance.#field_ident.clone())
                            }
                        }
                        "f64" => {
                            quote! {
                                .with_number_list_field(stringify!(#field_ident), |instance| instance.#field_ident.clone())
                            }
                        }
                        "bool" => {
                            quote! {
                                .with_boolean_list_field(stringify!(#field_ident), |instance| instance.#field_ident.clone())
                            }
                        }
                        "DateTime" => {
                            if vec_type.sub_ty.as_ref().is_some_and(|sub| sub.ty == "Utc") {
                                quote! {
                                    .with_datetime_list_field(stringify!(#field_ident), |instance| instance.#field_ident.clone())
                                }
                            } else {
                                panic!("DateTime fields have to be DateTime<Utc>");
                            }
                        }
                        "Vec" => {
                            let vec_type = vec_type.sub_ty.as_ref().unwrap();

                            match vec_type.ty.to_string().as_str() {
                                "u8" => {
                                    quote! {
                                        .with_raw_list_field(stringify!(#field_ident), |instance| instance.#field_ident.clone())
                                    }
                                }
                                _ => {
                                    panic!(
                                        "Invalid type. Only supported types are: String, f64, Vec<u8> and DateTime<Utc>"
                                    );
                                }
                            }
                        }
                        _ => {
                            panic!(
                                "Invalid type. Only supported types are: String, f64, Vec<u8> and DateTime<Utc>"
                            );
                        }
                    }
                }
                _ => {
                    let ty = &option_type.ty;

                    quote! {
                        .with_sub_field(
                            stringify!(#field_ident),
                            &<#ty as expression::schema::SchemaTarget<#ty>>::build_schema(),
                            |instance| instance.#field_ident.as_ref()
                        )
                    }
                }
            }
        }
        _ => {
            let ty = base_type.ty;

            quote! {
                .with_sub_field(
                    stringify!(#field_ident),
                    &<#ty as expression::schema::SchemaTarget<#ty>>::build_schema(),
                    |instance| Some(&instance.#field_ident)
                )
            }
        }
    }
}

#[proc_macro_derive(AutoSchema)]
pub fn main(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let syn::Data::Struct(data) = input.data else {
        panic!("The AutoSchema derive macro can only be used with structs");
    };

    let struct_ident = &input.ident;

    let mut builder = quote! {
        expression::schema::SchemaBuilder::<#struct_ident>::new()
    };

    for field in data.fields.iter() {
        let builder_field = gen_builder_field(field);

        builder = quote! {
            #builder
            #builder_field
        };
    }

    let expanded = quote! {
        impl #struct_ident {
            fn get_engine() -> expression::engine::Engine<#struct_ident> {
                expression::engine::Engine::new(
                    <#struct_ident as expression::schema::SchemaTarget<#struct_ident>>::build_schema()
                )
            }
        }

        impl expression::schema::SchemaTarget<#struct_ident> for #struct_ident {
            fn build_schema() -> expression::schema::Schema<#struct_ident> {
                #builder.build()
            }
        }
    };

    expanded.into()
}
