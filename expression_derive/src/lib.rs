#![feature(log_syntax)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, PathArguments, parse_macro_input};

// Simplified and easily queriable type struct
// Only maps types that don't have any form of arguments,
// and angle bracketed types that have only one argument
struct Type {
    pub ty: String,
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
            ty: segment.ident.to_string(),
            sub_ty,
        })
    }
}

#[proc_macro_derive(AutoSchema)]
pub fn main(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let syn::Data::Struct(data) = input.data else {
        return (quote! {
            compile_error!("The AutoSchema derive macro can only be used with structs");
        })
        .into();
    };

    let struct_ident = &input.ident;

    let mut builder = quote! {
        expression::schema::SchemaBuilder::<#struct_ident>::new()
    };

    for field in data.fields.iter() {
        let field_ident = match &field.ident {
            Some(ident) => ident,
            None => panic!("Fields must have names"),
        };

        let base_type: Type = match (&field.ty).try_into() {
            Ok(ty) => ty,
            Err(_) => {
                builder = quote! {
                    compile_error!("An error happened when parsing a type - better message TODO");
                };
                break;
            }
        };

        match base_type.ty.as_str() {
            "String" => {
                builder = quote! {
                    #builder
                    .with_string_field(stringify!(#field_ident), |instance| Some(instance.#field_ident.clone()))
                };
            }
            "f64" => {
                builder = quote! {
                    #builder
                    .with_number_field(stringify!(#field_ident), |instance| Some(instance.#field_ident))
                };
            }
            "bool" => {
                builder = quote! {
                    #builder
                    .with_boolean_field(stringify!(#field_ident), |instance| Some(instance.#field_ident))
                };
            }
            "DateTime" => {
                if base_type.sub_ty.as_ref().is_some_and(|sub| sub.ty == "Utc") {
                    builder = quote! {
                        #builder
                        .with_datetime_field(stringify!(#field_ident), |instance| Some(instance.#field_ident))
                    };
                } else {
                    builder = quote! {
                        compile_error!("DateTime fields have to be DateTime<Utc>");
                    };
                    break;
                }
            }
            "Vec" => {
                let vec_type = base_type.sub_ty.as_ref().unwrap();

                match vec_type.ty.as_str() {
                    "u8" => {
                        builder = quote! {
                            #builder
                            .with_raw_field(stringify!(#field_ident), |instance| Some(instance.#field_ident.clone()))
                        };
                    }
                    "String" => {
                        builder = quote! {
                            #builder
                            .with_string_list_field(stringify!(#field_ident), |instance| Some(instance.#field_ident.clone()))
                        };
                    }
                    "f64" => {
                        builder = quote! {
                            #builder
                            .with_number_list_field(stringify!(#field_ident), |instance| Some(instance.#field_ident.clone()))
                        };
                    }
                    "bool" => {
                        builder = quote! {
                            #builder
                            .with_boolean_list_field(stringify!(#field_ident), |instance| Some(instance.#field_ident.clone()))
                        };
                    }
                    "DateTime" => {
                        if vec_type.sub_ty.as_ref().is_some_and(|sub| sub.ty == "Utc") {
                            builder = quote! {
                                #builder
                                .with_datetime_list_field(stringify!(#field_ident), |instance| Some(instance.#field_ident.clone()))
                            };
                        } else {
                            builder = quote! {
                                compile_error!("DateTime fields have to be DateTime<Utc>");
                            };
                            break;
                        }
                    }
                    "Vec" => {
                        let vec_type = vec_type.sub_ty.as_ref().unwrap();

                        match vec_type.ty.as_str() {
                            "u8" => {
                                builder = quote! {
                                    #builder
                                    .with_raw_list_field(stringify!(#field_ident), |instance| Some(instance.#field_ident.clone()))
                                };
                            }
                            _ => {
                                builder = quote! {
                                    compile_error!("Invalid type - better message todo 4");
                                };
                                break;
                            }
                        }
                    }
                    _ => {
                        builder = quote! {
                            compile_error!("Invalid type - better message todo 2");
                        };
                        break;
                    }
                }
            }
            _ => {
                builder = quote! {
                    compile_error!("Invalid type - better message todo 1");
                }
            }
        }
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
