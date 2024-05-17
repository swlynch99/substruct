use std::rc::Rc;

use heck::ToSnakeCase;
use indexmap::IndexMap;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

use crate::expr::Expr;

/// A single input argument to the `#[substruct]` attribute.
///
/// ```text
/// /// Some doc comment
/// #[doc = "or doc attribute"]
/// <expr>
/// ```
struct SubstructInputArg {
    docs: Vec<syn::Attribute>,
    expr: Expr,
}

impl Parse for SubstructInputArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = syn::Attribute::parse_outer(input)?;

        for attr in &attrs {
            if !attr.path().is_ident("doc") {
                return Err(syn::Error::new(
                    attr.span(),
                    "only #[doc] attributes are permitted within #[substruct] arguments",
                ));
            }
        }

        Ok(Self {
            docs: attrs,
            expr: input.parse()?,
        })
    }
}

#[derive(Default)]
struct SubstructInput {
    args: Punctuated<SubstructInputArg, syn::Token![,]>,
}

impl SubstructInput {
    pub fn matching(&self, ident: &syn::Ident) -> Option<&SubstructInputArg> {
        self.args.iter().find(|arg| arg.expr.evaluate(ident))
    }
}

impl Parse for SubstructInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            args: Punctuated::parse_terminated(input)?,
        })
    }
}

struct SubstructAttrInput {
    expr: Expr,
    _comma: syn::Token![,],
    meta: syn::Meta,
}

impl Parse for SubstructAttrInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            expr: input.parse()?,
            _comma: input.parse()?,
            meta: input.parse()?,
        })
    }
}

struct TopLevelArg {
    docs: Vec<syn::Attribute>,
}

struct Emitter<'a> {
    input: &'a syn::DeriveInput,

    /// Use indexmap so that structs are emitted in the order they are specified
    /// in the macro arguments.
    args: Rc<IndexMap<syn::Ident, TopLevelArg>>,

    errors: Vec<syn::Error>,

    tokens: TokenStream,
}

impl<'a> Emitter<'a> {
    pub fn from_input(input: &'a syn::DeriveInput, attr: SubstructInput) -> syn::Result<Self> {
        if let syn::Data::Enum(data) = &input.data {
            return Err(syn::Error::new(
                data.enum_token.span,
                "#[substruct] does not support enums"
            ))
        }
        
        let mut errors = Vec::new();
        let mut args: IndexMap<syn::Ident, TopLevelArg> = attr
            .args
            .into_iter()
            .filter_map(|arg| match arg.expr {
                Expr::Ident(ident) => Some((ident.clone(), TopLevelArg { docs: arg.docs })),
                expr => {
                    errors.push(syn::Error::new_spanned(
                    expr,
                    "expressions are not permitted within a struct-level #[substruct] annotation",
                ));
                    None
                }
            })
            .collect();

        if !args.contains_key(&input.ident) {
            args.insert(input.ident.clone(), TopLevelArg { docs: Vec::new() });
        }

        Ok(Self {
            input,
            args: Rc::new(args),
            errors,
            tokens: TokenStream::new(),
        })
    }

    pub fn emit(mut self) -> TokenStream {
        let args = self.args.clone();
        for name in args.keys() {
            self.emit_struct(name);
        }

        for error in self.errors.drain(..) {
            self.tokens.extend(error.into_compile_error())
        }

        self.tokens
    }

    fn emit_struct(&mut self, name: &syn::Ident) {
        let tla = match self.args.get(name) {
            Some(tla) => tla,
            None => panic!("Attempted to emit struct `{name}` with no corresponding entry in the top-level arguments")
        };

        let mut input = self.input.clone();
        input.ident = name.clone();

        if !tla.docs.is_empty() {
            input.attrs.retain(|attr| !attr.path().is_ident("doc"));
            input.attrs.extend_from_slice(&tla.docs);
        }

        self.filter_attrs(&mut input.attrs, name);

        match &mut input.data {
            syn::Data::Enum(_) => return,
            // syn::Data::Enum(_) => panic!("Attempted to emit substruct on an enum"),
            syn::Data::Struct(data) => match &mut data.fields {
                syn::Fields::Named(fields) => self.filter_fields_named(fields, name),
                syn::Fields::Unnamed(fields) => self.filter_fields_unnamed(fields, name),
                syn::Fields::Unit => (),
            },
            syn::Data::Union(data) => self.filter_fields_named(&mut data.fields, name),
        };

        input.to_tokens(&mut self.tokens);

        if input.ident != self.input.ident {
            self.emit_conversions(&input);
        }
    }

    fn emit_conversions(&mut self, substruct: &syn::DeriveInput) {
        if !self.errors.is_empty() {
            return;
        }

        let original = &self.input.ident;
        let name = &substruct.ident;
        let (impl_generics, ty_generics, where_clause) = substruct.generics.split_for_impl();

        let method = syn::Ident::new(
            &format!("into_{}", self.input.ident.to_string().to_snake_case()),
            Span::call_site(),
        );
        let doc: syn::Attribute = syn::parse_quote!(
            #[doc = concat!("Convert `self` into a [`", stringify!(#original), "`].")]
        );

        let fields = match &self.input.data {
            syn::Data::Enum(_) => panic!("Attempted to emit conversions for an enum"),
            // Emitting conversions for an enum doesn't make sense
            syn::Data::Union(_) => return,
            // Unit structs have no fields and so they have no conversions
            syn::Data::Struct(data) if matches!(data.fields, syn::Fields::Unit) => return,
            syn::Data::Struct(data) => &data.fields,
        };

        let mut included = IndexMap::new();
        let mut excluded = IndexMap::new();

        for (index, mut field) in fields.iter().cloned().enumerate() {
            let filter = self.filter_field(&mut field, &substruct.ident);
            let id = match field.ident {
                Some(ident) => IdentOrIndex::Ident(ident),
                None => IdentOrIndex::Index(index),
            };

            if filter {
                included.insert(id, field.ty);
            } else {
                excluded.insert(id, field.ty);
            }
        }

        let args: Vec<_> = excluded.keys().cloned().map(|key| key.into_ident()).collect();
        let types: Vec<_> = excluded.values().collect();

        let inc_dst: Vec<_> = included.keys().collect();
        // Renumber source indexes so they refer to the smaller struct
        let inc_src: Vec<_> = included
            .keys()
            .enumerate()
            .map(|(index, name)| match name {
                IdentOrIndex::Ident(ident) => IdentOrIndex::Ident(ident.clone()),
                IdentOrIndex::Index(_) => IdentOrIndex::Index(index),
            })
            .collect();
        let exc: Vec<_> = excluded.keys().collect();

        self.tokens.extend(quote::quote! {
            impl #impl_generics #name #ty_generics
            #where_clause
            {
                #doc
                pub fn #method(self, #( #args: #types, )*) -> #original #ty_generics {
                    #original {
                        #( #inc_dst: self.#inc_src, )*
                        #( #exc: #args, )*
                    }
                }
            }
        });

        self.tokens.extend(quote::quote! {
            impl #impl_generics From<#original #ty_generics> for #name #ty_generics
            #where_clause
            {
                fn from(value: #original #ty_generics) -> Self {
                    Self {
                        #( #inc_src: value.#inc_dst, )*
                    }
                }
            }
        });

        if excluded.is_empty() {
            self.tokens.extend(quote::quote! {
                impl #impl_generics From<#name #ty_generics> for #original #ty_generics
                #where_clause
                {
                    fn from(value: #name #ty_generics) -> Self {
                        value.#method()
                    }
                }
            })
        }
    }

    fn filter_fields_named(&mut self, fields: &mut syn::FieldsNamed, name: &syn::Ident) {
        fields.named = std::mem::take(&mut fields.named)
            .into_pairs()
            .filter_map(|mut pair| match self.filter_field(pair.value_mut(), name) {
                true => Some(pair),
                false => None,
            })
            .collect();
    }

    fn filter_fields_unnamed(&mut self, fields: &mut syn::FieldsUnnamed, name: &syn::Ident) {
        fields.unnamed = std::mem::take(&mut fields.unnamed)
            .into_pairs()
            .filter_map(|mut pair| match self.filter_field(pair.value_mut(), name) {
                true => Some(pair),
                false => None,
            })
            .collect();
    }

    fn filter_field(&mut self, field: &mut syn::Field, name: &syn::Ident) -> bool {
        let substruct: Vec<_> = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("substruct"))
            .collect();

        let mut substruct = match substruct {
            substruct if substruct.is_empty() => Default::default(),
            substruct => {
                let args: Option<SubstructInput> = match substruct[0].parse_args() {
                    Ok(args) => Some(args),
                    Err(e) => {
                        self.errors.push(e);
                        None
                    }
                };

                for attr in &substruct[1..] {
                    self.errors.push(syn::Error::new_spanned(
                        attr,
                        "only one #[substruct] attribute is allowed on a field",
                    ));
                }

                args.unwrap_or_default()
            }
        };

        substruct.args.push(SubstructInputArg {
            docs: Vec::new(),
            expr: Expr::Ident(self.input.ident.clone()),
        });

        let arg = match substruct.matching(name) {
            Some(arg) => arg,
            None => return false,
        };

        self.filter_attrs(&mut field.attrs, name);

        if !arg.docs.is_empty() {
            field.attrs.retain(|attr| !attr.path().is_ident("doc"));
            field.attrs.extend_from_slice(&arg.docs);
        }

        true
    }

    fn filter_attrs(&mut self, attrs: &mut Vec<syn::Attribute>, name: &syn::Ident) {
        attrs.retain_mut(|attr| {
            let path = attr.path();

            if path.is_ident("substruct") {
                return false;
            }

            if !path.is_ident("substruct_attr") {
                return true;
            }

            let args: SubstructAttrInput = match attr.parse_args() {
                Ok(args) => args,
                Err(e) => {
                    self.errors.push(e);
                    return false;
                }
            };

            if args.expr.evaluate(name) {
                attr.meta = args.meta;
                true
            } else {
                false
            }
        })
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let input: syn::DeriveInput = syn::parse2(item)?;
    let args: SubstructInput = syn::parse2(attr)?;

    Ok(Emitter::from_input(&input, args)?.emit())
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
enum IdentOrIndex {
    Ident(syn::Ident),
    Index(usize),
}

impl IdentOrIndex {
    fn into_ident(self) -> syn::Ident {
        match self {
            Self::Ident(ident) => ident,
            Self::Index(index) => syn::Ident::new(&format!("arg{index}"), Span::call_site()),
        }
    }
}

impl ToTokens for IdentOrIndex {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::Index(index) => {
                syn::LitInt::new(&index.to_string(), Span::call_site()).to_tokens(tokens)
            }
        }
    }
}
