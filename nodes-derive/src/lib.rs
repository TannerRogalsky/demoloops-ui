#![allow(unused)]

use syn::{Data, DeriveInput};

#[proc_macro_derive(FromAny)]
#[proc_macro_error::proc_macro_error]
pub fn derive_from_any(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(input);

    let output = match &input.data {
        Data::Struct(_) | Data::Union(_) => {
            proc_macro_error::abort!(input.ident, "Only enums are supported")
        }
        Data::Enum(data) => {
            let all = data
                .variants
                .iter()
                .map(|variant| {
                    assert_eq!(1, variant.fields.len());
                    let field = variant.fields.iter().next().unwrap();
                    (&variant.ident, &field.ty)
                })
                .collect::<Vec<_>>();

            let fields = all.iter().map(|(_, field)| field);
            let downcasts = all.iter().map(|(variant, field)| {
                quote::quote! {
                    match <#field>::downcast(v) {
                        Ok(v) => return Ok(Self::#variant(v)),
                        Err(err) => err,
                    }
                }
            });

            let type_id_count = all.len() * 3;
            let type_ids = all.iter().map(|(variant, field)| {
                quote::quote! {
                    #[allow(non_snake_case)]
                    let #variant = <#field>::types();
                }
            });
            let type_ids_splat = all.iter().map(|(variant, _)| {
                quote::quote! {
                    #variant[0], #variant[1], #variant[2]
                }
            });

            let ident = &input.ident;
            quote::quote! {
                impl crate::FromAny for #ident {
                    fn from_any(inputs: &mut Vec<Box<dyn std::any::Any>>) -> Result<Self, ()> {

                    }
                    fn types(names: &'static [&str]) -> Vec<InputGroup<'static>> {
                        let mut acc
                    };
                }
            }
            // quote::quote! {
            //     impl #ident {
            //         fn is(v: &dyn std::any::Any) -> bool {
            //             #(<#fields>::is(v)) ||*
            //         }
            //
            //         fn downcast(v: Box<dyn std::any::Any>) -> Result<Self, Box<dyn std::any::Any>> {
            //             #(let v = #downcasts);*;
            //             Err(v)
            //         }
            //
            //         fn type_ids() -> [std::any::TypeId; #type_id_count] {
            //             #(#type_ids);*;
            //             [#(#type_ids_splat),*]
            //         }
            //     }
            // }
        }
    };

    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(InputComponent)]
#[proc_macro_error::proc_macro_error]
pub fn derive_from_input_component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(input);

    let output = match &input.data {
        Data::Struct(_) | Data::Union(_) => {
            proc_macro_error::abort!(input.ident, "Only enums are supported")
        }
        Data::Enum(data) => {
            let all = data
                .variants
                .iter()
                .map(|variant| {
                    assert_eq!(1, variant.fields.len());
                    let field = variant.fields.iter().next().unwrap();
                    (&variant.ident, &field.ty)
                })
                .collect::<Vec<_>>();

            let fields = all.iter().map(|(_, field)| field);
            let downcasts = all.iter().map(|(variant, field)| {
                quote::quote! {
                    match <#field>::downcast(v) {
                        Ok(v) => return Ok(Self::#variant(v)),
                        Err(err) => err,
                    }
                }
            });

            let type_id_count = all.len() * 3;
            let type_ids = all.iter().map(|(variant, field)| {
                quote::quote! {
                    #[allow(non_snake_case)]
                    let #variant = <#field>::types();
                }
            });
            let type_ids_splat = all.iter().map(|(variant, _)| {
                quote::quote! {
                    #variant[0], #variant[1], #variant[2]
                }
            });

            let ident = &input.ident;
            quote::quote! {
                impl crate::FromAny for #ident {
                    fn from_any(inputs: &mut Vec<Box<dyn std::any::Any>>) -> Result<Self, ()> {

                    }
                    fn types(names: &'static [&str]) -> Vec<InputGroup<'static>> {
                        let mut acc
                    };
                }
            }
            // quote::quote! {
            //     impl #ident {
            //         fn is(v: &dyn std::any::Any) -> bool {
            //             #(<#fields>::is(v)) ||*
            //         }
            //
            //         fn downcast(v: Box<dyn std::any::Any>) -> Result<Self, Box<dyn std::any::Any>> {
            //             #(let v = #downcasts);*;
            //             Err(v)
            //         }
            //
            //         fn type_ids() -> [std::any::TypeId; #type_id_count] {
            //             #(#type_ids);*;
            //             [#(#type_ids_splat),*]
            //         }
            //     }
            // }
        }
    };

    proc_macro::TokenStream::from(output)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2, 1 + 1);
    }
}
