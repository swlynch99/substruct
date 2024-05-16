use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

pub(crate) enum Expr {
    Ident(syn::Ident),
    Not(NotExpr),
    All(AllExpr),
    Any(AnyExpr),
}

impl Expr {
    pub fn evaluate(&self, ident: &syn::Ident) -> bool {
        match self {
            Self::Ident(lit) => ident == lit,
            Self::Not(e) => e.evaluate(ident),
            Self::Any(e) => e.evaluate(ident),
            Self::All(e) => e.evaluate(ident),
        }
    }
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if !input.peek2(syn::token::Paren) {
            return Ok(Self::Ident(input.parse()?));
        }

        let ident: syn::Ident = input.fork().parse()?;

        match () {
            _ if ident == "not" => input.parse().map(Self::Not),
            _ if ident == "any" => input.parse().map(Self::Any),
            _ if ident == "all" => input.parse().map(Self::All),
            _ => Err(syn::Error::new(
                ident.span(),
                format!("unexpected operator `{ident}`, expected `not`, `any`, or `all`"),
            )),
        }
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::Not(e) => e.to_tokens(tokens),
            Self::All(e) => e.to_tokens(tokens),
            Self::Any(e) => e.to_tokens(tokens),
        }
    }
}

pub(crate) struct NotExpr {
    pub ident: syn::Ident,
    pub paren: syn::token::Paren,
    pub expr: Box<Expr>,
}

impl NotExpr {
    pub fn evaluate(&self, ident: &syn::Ident) -> bool {
        !self.expr.evaluate(ident)
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

pub(crate) struct AnyExpr {
    pub ident: syn::Ident,
    pub paren: syn::token::Paren,
    pub exprs: Punctuated<Expr, syn::Token![,]>,
}

impl AnyExpr {
    pub fn evaluate(&self, ident: &syn::Ident) -> bool {
        self.exprs.iter().any(|e| e.evaluate(ident))
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

pub(crate) struct AllExpr {
    pub ident: syn::Ident,
    pub paren: syn::token::Paren,
    pub exprs: Punctuated<Expr, syn::Token![,]>,
}

impl AllExpr {
    pub fn evaluate(&self, ident: &syn::Ident) -> bool {
        self.exprs.iter().all(|e| e.evaluate(ident))
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
