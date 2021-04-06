#![allow(unused)]

use syn::{Data, DeriveInput, Type};

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

            let is_option = types.iter().map(|t| match t {
                Type::Path(path) => path.path.segments.last().unwrap().ident == "Option",
                _ => false,
            });

            let downcasts = types.iter().zip(is_option).map(|(ty, is_option)| {
                if is_option {
                    // if optional inputs are ever changed to not require a box then this should
                    // change from `ok` to unwrap
                    quote::quote! {
                        inputs.next().and_then(|ty| <#ty>::downcast(ty).ok()).flatten()
                    }
                } else {
                    quote::quote! {
                        <#ty>::downcast(inputs.next().unwrap()).unwrap()
                    }
                }
            });

            quote::quote! {
                impl #impl_generics ::nodes::FromAnyProto for #ident #ty_generics #where_clause {
                    fn from_any(inputs: ::nodes::InputStack<'_, Box<dyn std::any::Any>>) -> Result<Self, ()> {
                        use ::nodes::InputComponent;

                        let required = [#(<#types>::is_optional(),)*];
                        let required_count = required.iter().copied().filter(|v| !*v).count();

                        if inputs.as_slice().len() < required_count {
                            eprintln!("{} < {}", inputs.as_slice().len(), required_count);
                            return Err(());
                        }

                        let mut is_optional = required.iter().copied();
                        let mut checker = inputs.deref_iter();
                        #({
                            let is_optional = is_optional.next().unwrap();
                            let is_type = checker.next().map(|v| <#types>::is(v)).unwrap_or(false);
                            if !is_optional && !is_type {
                                eprintln!("Type Mismatch in {}: {}", std::any::type_name::<#ident>(), std::any::type_name::<#types>());
                                return Err(());
                            }
                        })*

                        let mut inputs = inputs.consume();
                        Ok(#ident {#(
                            #fields: #downcasts,
                            // #fields: {
                            //     // let v = conditional_downcast::<#types>(inputs.next());
                            //     let v = <#types>::downcast(inputs.next().unwrap()).unwrap();
                            //     if !<#types>::is_optional() {
                            //         v
                            //     } else {
                            //         Some(v)
                            //     }
                            // },
                        )*})
                    }
                    fn possible_inputs(names: &'static [&str]) -> ::nodes::PossibleInputs<'static> {
                        use ::nodes::{Itertools, InputComponent};
                        let groups = std::array::IntoIter::new([#(<#types>::type_ids()),*])
                            .multi_cartesian_product()
                            .map(|types| ::nodes::InputGroup {
                                info: std::array::IntoIter::new([#((std::any::type_name::<#types>(), <#types>::is_optional())),*])
                                .zip(names.iter().copied().zip(types))
                                .map(|((ty_name, optional), (name, type_id))| ::nodes::InputInfo {
                                    name: name.into(),
                                    ty_name,
                                    type_id,
                                    optional,
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

            let fields = all.iter().map(|(_, field)| field);

            let downcasts = all.iter().map(|(variant, field)| {
                quote::quote! {
                    if let Ok(v) = <#field as ::nodes::FromAnyProto>::from_any(inputs.sub(..)) {
                        return Ok(Self::#variant(v));
                    }
                }
            });

            quote::quote! {
                impl #impl_generics ::nodes::FromAnyProto for #ident #ty_generics #where_clause {
                    fn from_any(mut inputs: ::nodes::InputStack<'_, Box<dyn std::any::Any>>) -> Result<Self, ()> {
                        #(#downcasts);*
                        Err(())
                    }
                    fn possible_inputs(names: &'static [&str]) -> ::nodes::PossibleInputs<'static> {
                        use ::nodes::Itertools;
                        let groups = std::array::IntoIter::new([#(<#fields>::possible_inputs(names)), *])
                            .flat_map(|p| p.groups.into_owned().into_iter())
                            .collect::<Vec<::nodes::InputGroup>>();
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
                impl #impl_generics ::nodes::InputComponent for #ident #ty_generics #where_clause {
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
                impl #impl_generics ::nodes::InputComponent for #ident #ty_generics #where_clause {
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
