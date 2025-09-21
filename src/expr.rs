use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

#[derive(Clone)]
pub(crate) enum Expr {
    Ident(syn::Ident),
    IdentWithTransform(IdentWithTransformExpr),
    Not(NotExpr),
    All(AllExpr),
    Any(AnyExpr),
}

impl Expr {
    pub fn evaluate(&self, ident: &syn::Ident) -> bool {
        match self {
            Self::Ident(lit) => ident == lit,
            Self::IdentWithTransform(e) => e.evaluate(ident),
            Self::Not(e) => e.evaluate(ident),
            Self::Any(e) => e.evaluate(ident),
            Self::All(e) => e.evaluate(ident),
        }
    }

    pub fn get_transform(&self, ident: &syn::Ident) -> Option<&TransformType> {
        match self {
            Self::Ident(_) => None,
            Self::IdentWithTransform(e) => e.get_transform(ident),
            Self::Not(e) => e.get_transform(ident),
            Self::Any(e) => e.get_transform(ident),
            Self::All(e) => e.get_transform(ident),
        }
    }
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // If an identifier is followed by parentheses then it may be an operator
        // like `not(...)` / `any(...)` / `all(...)` or a transform spec like
        // `Name(unwrap)` / `Name(try_into = Type)`. Inspect without consuming
        // by using a fork and then dispatch accordingly.
        if input.peek(syn::Ident) && input.peek2(syn::token::Paren) {
            let ident: syn::Ident = input.fork().parse()?;
            match ident.to_string().as_str() {
                "not" => input.parse().map(Self::Not),
                "any" => input.parse().map(Self::Any),
                "all" => input.parse().map(Self::All),
                _ => {
                    // Parse as IdentWithTransform for struct names with transforms like A(unwrap)
                    input.parse().map(Self::IdentWithTransform)
                }
            }
        } else {
            // Plain identifier (no parentheses following) is just an Ident
            Ok(Self::Ident(input.parse()?))
        }
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::IdentWithTransform(e) => e.to_tokens(tokens),
            Self::Not(e) => e.to_tokens(tokens),
            Self::All(e) => e.to_tokens(tokens),
            Self::Any(e) => e.to_tokens(tokens),
        }
    }
}

#[derive(Clone)]
pub(crate) struct IdentWithTransformExpr {
    pub(crate) ident: syn::Ident,
    pub(crate) paren: syn::token::Paren,
    pub(crate) transform: TransformType,
}

#[derive(Clone)]
pub(crate) enum TransformType {
    Unwrap,
    TryInto(Box<syn::Type>),
}

impl IdentWithTransformExpr {
    pub fn evaluate(&self, ident: &syn::Ident) -> bool {
        self.ident == *ident
    }

    pub fn get_transform(&self, ident: &syn::Ident) -> Option<&TransformType> {
        if self.ident == *ident {
            Some(&self.transform)
        } else {
            None
        }
    }
}

impl Parse for IdentWithTransformExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let ident = input.parse()?;
        let paren = syn::parenthesized!(content in input);

        let transform = if content.peek(syn::Ident) {
            let transform_ident: syn::Ident = content.parse()?;

            if transform_ident == "unwrap" {
                TransformType::Unwrap
            } else if transform_ident == "try_into" {
                // Parse "try_into = Type"
                content.parse::<syn::Token![=]>()?;
                let target_type: syn::Type = content.parse()?;
                TransformType::TryInto(Box::new(target_type))
            } else {
                return Err(syn::Error::new_spanned(
                    &transform_ident,
                    "unknown transformation type, expected 'unwrap' or 'try_into'",
                ));
            }
        } else {
            return Err(syn::Error::new(
                content.span(),
                "expected transformation type ('unwrap' or 'try_into = Type')",
            ));
        };

        Ok(Self {
            ident,
            paren,
            transform,
        })
    }
}

impl ToTokens for IdentWithTransformExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.paren.surround(tokens, |tokens| match &self.transform {
            TransformType::Unwrap => {
                tokens.extend(quote::quote! { unwrap });
            }
            TransformType::TryInto(ty) => {
                tokens.extend(quote::quote! { try_into = #ty });
            }
        });
    }
}

#[derive(Clone)]
pub(crate) struct NotExpr {
    pub(crate) ident: syn::Ident,
    pub(crate) paren: syn::token::Paren,
    pub(crate) expr: Box<Expr>,
}

impl NotExpr {
    pub fn evaluate(&self, ident: &syn::Ident) -> bool {
        !self.expr.evaluate(ident)
    }

    pub fn get_transform(&self, ident: &syn::Ident) -> Option<&TransformType> {
        self.expr.get_transform(ident)
    }
}

impl Parse for NotExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Self {
            ident: input.parse()?,
            paren: syn::parenthesized!(content in input),
            expr: content.parse()?,
        })
    }
}

impl ToTokens for NotExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.paren
            .surround(tokens, |tokens| self.expr.to_tokens(tokens));
    }
}

#[derive(Clone)]
pub(crate) struct AnyExpr {
    pub(crate) ident: syn::Ident,
    pub(crate) paren: syn::token::Paren,
    pub(crate) exprs: Punctuated<Expr, syn::Token![,]>,
}

impl AnyExpr {
    pub fn evaluate(&self, ident: &syn::Ident) -> bool {
        self.exprs.iter().any(|e| e.evaluate(ident))
    }

    pub fn get_transform(&self, ident: &syn::Ident) -> Option<&TransformType> {
        self.exprs.iter().find_map(|e| e.get_transform(ident))
    }
}

impl Parse for AnyExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        let ident: syn::Ident = input.parse()?;
        if ident != "any" {
            return Err(syn::Error::new(
                ident.span(),
                format_args!("expected `any`, got `{ident}` instead"),
            ));
        }

        Ok(Self {
            ident,
            paren: syn::parenthesized!(content in input),
            exprs: Punctuated::parse_terminated(&content)?,
        })
    }
}

impl ToTokens for AnyExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.paren
            .surround(tokens, |tokens| self.exprs.to_tokens(tokens));
    }
}

#[derive(Clone)]
pub(crate) struct AllExpr {
    pub(crate) ident: syn::Ident,
    pub(crate) paren: syn::token::Paren,
    pub(crate) exprs: Punctuated<Expr, syn::Token![,]>,
}

impl AllExpr {
    pub fn evaluate(&self, ident: &syn::Ident) -> bool {
        self.exprs.iter().all(|e| e.evaluate(ident))
    }

    pub fn get_transform(&self, ident: &syn::Ident) -> Option<&TransformType> {
        self.exprs.iter().find_map(|e| e.get_transform(ident))
    }
}

impl Parse for AllExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        let ident: syn::Ident = input.parse()?;
        if ident != "all" {
            return Err(syn::Error::new(
                ident.span(),
                format_args!("expected `all`, got `{ident}` instead"),
            ));
        }

        Ok(Self {
            ident,
            paren: syn::parenthesized!(content in input),
            exprs: Punctuated::parse_terminated(&content)?,
        })
    }
}

impl ToTokens for AllExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.paren
            .surround(tokens, |tokens| self.exprs.to_tokens(tokens));
    }
}
