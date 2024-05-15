use std::collections::HashSet;

use heck::ToSnakeCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::parse::Parse;
use syn::punctuated::Punctuated;

struct Args(Punctuated<Ident, syn::Token![,]>);

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(Punctuated::parse_terminated(input)?))
    }
}

struct Metadata {
    fields: HashSet<(usize, Ident)>,
}

impl Metadata {
    fn from_input(args: &Args, data: &syn::DataStruct) -> syn::Result<Self> {
        let mut fields = HashSet::new();
        let valid: HashSet<Ident> = args.0.iter().cloned().collect();

        for (idx, field) in data.fields.iter().enumerate() {
            let attr = match field
                .attrs
                .iter()
                .find(|attr| attr.meta.path().is_ident("substruct"))
            {
                Some(attr) => attr,
                None => continue,
            };

            let list = attr.meta.require_list()?;
            let args: Args = syn::parse2(list.tokens.clone())?;

            for substruct in args.0 {
                if !valid.contains(&substruct) {
                    return Err(syn::Error::new(
                        substruct.span(),
                        format!("struct name `{substruct}` does not appear the the top-level list of structs to create")
                    ));
                }

                fields.insert((idx, substruct));
            }
        }

        Ok(Self { fields })
    }

    fn substruct_includes_field(&self, substruct: &Ident, field: usize) -> bool {
        self.fields.contains(&(field, substruct.clone()))
    }

    fn emit_substruct(
        &self,
        mut input: syn::DeriveInput,
        name: &Ident,
    ) -> syn::Result<TokenStream> {
        let original = std::mem::replace(&mut input.ident, name.clone());
        let data = match &mut input.data {
            syn::Data::Struct(data) => data,
            _ => unreachable!(),
        };

        let mut excluded = Vec::new();
        let mut incindices = Vec::new();

        data.fields = match &mut data.fields {
            syn::Fields::Unit => syn::Fields::Unit,
            syn::Fields::Named(fields) => {
                let mut included = Punctuated::new();

                for (idx, field) in std::mem::take(&mut fields.named).into_iter().enumerate() {
                    if self.substruct_includes_field(name, idx) {
                        included.push(field);
                        incindices.push(idx);
                    } else {
                        excluded.push((idx, field));
                    }
                }

                syn::Fields::Named(syn::FieldsNamed {
                    brace_token: fields.brace_token,
                    named: included,
                })
            }
            syn::Fields::Unnamed(fields) => {
                let mut included = Punctuated::new();

                for (idx, field) in std::mem::take(&mut fields.unnamed).into_iter().enumerate() {
                    if self.substruct_includes_field(name, idx) {
                        included.push(field);
                        incindices.push(idx);
                    } else {
                        excluded.push((idx, field));
                    }
                }

                syn::Fields::Unnamed(syn::FieldsUnnamed {
                    paren_token: fields.paren_token,
                    unnamed: included,
                })
            }
        };

        let method = syn::Ident::new(
            &format!("into_{}", original.to_string().to_snake_case()),
            Span::call_site(),
        );

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
        let extra_impl = match &data.fields {
            syn::Fields::Unit => quote!(),
            syn::Fields::Named(fields) => {
                let names: Vec<_> = excluded
                    .iter()
                    .map(|(_, field)| field.ident.as_ref().unwrap())
                    .collect();
                let types: Vec<_> = excluded.iter().map(|(_, field)| &field.ty).collect();
                let existing: Vec<_> = fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap())
                    .collect();

                quote! {
                    impl #impl_generics #name #ty_generics
                    #where_clause
                    {
                        pub fn #method(self, #( #names: #types, )*) -> #original #ty_generics {
                            #original {
                                #( #names, )*
                                #( #existing: self.#existing, )*
                            }
                        }
                    }

                    impl #impl_generics From<#original #ty_generics> for #name #ty_generics
                    #where_clause
                    {
                        fn from(value: #original #ty_generics) -> Self {
                            Self {
                                #( #existing: value.#existing, )*
                            }
                        }
                    }
                }
            }
            syn::Fields::Unnamed(_) => {
                let existing_src: Vec<_> = (0..incindices.len())
                    .map(|idx| syn::LitInt::new(&idx.to_string(), Span::call_site()))
                    .collect();
                let existing_dst: Vec<_> = incindices
                    .iter()
                    .map(|idx| syn::LitInt::new(&idx.to_string(), Span::call_site()))
                    .collect();

                let excluded_dst: Vec<_> = excluded
                    .iter()
                    .map(|(idx, _)| syn::LitInt::new(&idx.to_string(), Span::call_site()))
                    .collect();

                let names: Vec<_> = excluded
                    .iter()
                    .map(|(idx, _)| syn::Ident::new(&format!("arg{idx}"), Span::call_site()))
                    .collect();
                let types: Vec<_> = excluded.iter().map(|(_, field)| &field.ty).collect();

                quote! {
                    impl #impl_generics #name #ty_generics
                    #where_clause
                    {
                        #[doc = concat!("Convert `self` into a `", stringify!(#original), "`.")]
                        pub fn #method(self, #( #names: #types, )*) -> #original #ty_generics {
                            #original {
                                #( #existing_dst: self.#existing_src, )*
                                #( #excluded_dst: #names, )*
                            }
                        }
                    }

                    impl #impl_generics From<#original #ty_generics> for #name #ty_generics
                    #where_clause
                    {
                        fn from(value: #original #ty_generics) -> Self {
                            Self {
                                #( #existing_src: value.#existing_dst, )*
                            }
                        }
                    }
                }
            }
        };

        Ok(quote! {
            #input
            #extra_impl
        })
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let mut input: syn::DeriveInput = syn::parse2(item)?;
    let args: Args = syn::parse2(attr)?;

    let data = match &mut input.data {
        syn::Data::Struct(data) => data,
        syn::Data::Enum(data) => {
            return Err(syn::Error::new(
                data.enum_token.span,
                "substruct does not support enums",
            ))
        }
        syn::Data::Union(data) => {
            return Err(syn::Error::new(
                data.union_token.span,
                "substruct does not support unions",
            ))
        }
    };

    let metadata = Metadata::from_input(&args, data)?;

    for field in &mut data.fields {
        field
            .attrs
            .retain(|attr| !attr.path().is_ident("substruct"));
    }

    let mut tokens = quote!(#input);
    for substruct in args.0 {
        tokens.extend(metadata.emit_substruct(input.clone(), &substruct)?);
    }

    Ok(tokens)
}
