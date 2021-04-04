#![allow(unused)]

use syn::{Data, DeriveInput};

#[proc_macro_derive(FromAnyProto)]
#[proc_macro_error::proc_macro_error]
pub fn derive_from_any(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(input);

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let output = match &input.data {
        Data::Union(_) => {
            proc_macro_error::abort!(input.ident, "Unions unsupported.")
        }
        Data::Struct(data) => {
            let fields = data
                .fields
                .iter()
                .map(|field| field.ident.as_ref().unwrap())
                .collect::<Vec<_>>();
            let types = data
                .fields
                .iter()
                .map(|field| &field.ty)
                .collect::<Vec<_>>();

            let count = data.fields.len();

            quote::quote! {
                impl #impl_generics  crate::FromAnyProto for #ident #ty_generics #where_clause {
                    fn from_any(inputs: InputStack<'_, Box<dyn std::any::Any>>) -> Result<Self, ()> {
                        if inputs.as_slice().len() < #count {
                            eprintln!("{} < {}", inputs.as_slice().len(), #count);
                            return Err(());
                        }

                        let mut checker = inputs.deref_iter();
                        #(if !<#types>::is(checker.next().unwrap()) {
                            eprintln!("{}", std::any::type_name::<#types>());
                            return Err(());
                        })*

                        let mut inputs = inputs.consume();
                        Ok(#ident {#(
                            #fields: <#types>::downcast(inputs.next().unwrap()).unwrap(),
                        )*})
                    }
                    fn possible_inputs(names: &'static [&str]) -> crate::PossibleInputs<'static> {
                        use crate::Itertools;
                        let groups = std::array::IntoIter::new([#(<#types as crate::InputComponent>::type_ids()),*])
                            .multi_cartesian_product()
                            .map(|types| InputGroup {
                                info: std::array::IntoIter::new([#(std::any::type_name::<#types>()),*])
                                .zip(names.iter().copied().zip(types))
                                .map(|(ty_name, (name, type_id))| crate::InputInfo {
                                    name: name.into(),
                                    ty_name,
                                    type_id,
                                })
                                .collect(),
                            })
                            .collect::<Vec<_>>();
                        PossibleInputs::new(groups)
                    }
                }
            }
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

            let count = data.variants.len();
            let fields = all.iter().map(|(_, field)| field);

            let downcasts = all.iter().map(|(variant, field)| {
                quote::quote! {
                    if let Ok(v) = <#field as crate::FromAnyProto>::from_any(inputs.sub(..)) {
                        return Ok(Self::#variant(v));
                    }
                }
            });

            quote::quote! {
                impl #impl_generics crate::FromAnyProto for #ident #ty_generics #where_clause {
                    fn from_any(mut inputs: InputStack<'_, Box<dyn std::any::Any>>) -> Result<Self, ()> {
                        #(#downcasts);*
                        Err(())
                    }
                    fn possible_inputs(names: &'static [&str]) -> crate::PossibleInputs<'static> {
                        use crate::Itertools;
                        let groups = std::array::IntoIter::new([#(<#fields>::possible_inputs(names)), *])
                            .flat_map(|p| p.groups.into_owned().into_iter())
                            .collect::<Vec<InputGroup>>();
                        PossibleInputs::new(groups)
                    }
                }
            }
        }
    };

    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(InputComponent)]
#[proc_macro_error::proc_macro_error]
pub fn derive_from_input_component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(input);

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let output = match &input.data {
        Data::Union(_) => {
            proc_macro_error::abort!(input.ident, "Unions unsupported.")
        }
        Data::Struct(data) => {
            quote::quote! {
                impl #impl_generics crate::InputComponent for #ident #ty_generics #where_clause {
                    fn is(v: &dyn std::any::Any) -> bool {
                        v.is::<#ident>()
                    }

                    fn type_ids() -> Vec<std::any::TypeId> {
                        vec![std::any::TypeId::of::<#ident>()]
                    }

                    fn downcast(v: Box<dyn std::any::Any>) -> Result<Self, Box<dyn std::any::Any>> {
                        v.downcast::<#ident>().map(|v| *v)
                    }
                }
            }
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

            let fields = all.iter().map(|(_, field)| field).collect::<Vec<_>>();
            let downcasts = all.iter().map(|(variant, field)| {
                quote::quote! {
                    let v = match <#field>::downcast(v) {
                        Ok(v) => return Ok(Self::#variant(v)),
                        Err(err) => err,
                    };
                }
            });

            quote::quote! {
                impl #impl_generics crate::InputComponent for #ident #ty_generics #where_clause {
                    fn is(v: &dyn std::any::Any) -> bool {
                        #(<#fields>::is(v)) ||* || v.is::<#ident>()
                    }

                    fn type_ids() -> Vec<std::any::TypeId> {
                        vec![#(std::any::TypeId::of::<#fields>()),*, std::any::TypeId::of::<#ident>()]
                    }

                    fn downcast(v: Box<dyn std::any::Any>) -> Result<Self, Box<dyn std::any::Any>> {
                        #(#downcasts);*
                        v.downcast::<#ident>().map(|v| *v)
                    }
                }
            }
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
