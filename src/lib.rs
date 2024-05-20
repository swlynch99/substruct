//! Substruct is a proc-macro to allow you to easily declare structs which are
//! subsets of another struct. It automatically creates conversion methods
//! between the parent and child structs and will propagate any derives and
//! attributes to the child struct.
//!
//! A basic use of substruct looks like this
//! ```
//! use substruct::substruct;
//!
//! #[substruct(SubQuery)]
//! #[derive(Clone, Debug, Eq, PartialEq)]
//! pub struct Query {
//!     #[substruct(SubQuery)]
//!     pub a: &'static str,
//!     pub b: usize
//! }
//!
//! let subquery = SubQuery { a: "query" };
//! let query = Query { a: "query", b: 5 };
//!
//! assert_eq!(subquery.into_query(5), query);
//! ```
//!
//! and that will expand to produce
//! ```
//! #[derive(Clone, Debug, Eq, PartialEq)]
//! pub struct Query {
//!     pub a: &'static str,
//!     pub b: usize
//! }
//!
//! #[derive(Clone, Debug, Eq, PartialEq)]
//! pub struct SubQuery {
//!     pub a: &'static str,
//! }
//! ```
//!
//! Substruct isn't just limited to creating a single child struct, you can use
//! it to create many at once:
//!
//! ```
//! # use substruct::substruct;
//! #[substruct(Vec2, Vec3)]
//! pub struct Vec4<T> {
//!     #[substruct(Vec2, Vec3)]
//!     pub x: T,
//!     
//!     #[substruct(Vec2, Vec3)]
//!     pub y: T,
//!
//!     #[substruct(Vec3)]
//!     pub z: T,
//!
//!     pub w: T,
//! }
//! ```
//!
//! **It is important that the `#[substruct]` attribute is placed before other
//! attributes.** The `#[substruct]` attribute macro can only see attributes
//! that come after it, with the exception of doc comments, so any attributes
//! that are evaluated before it will not be duplicated. While you can use this
//! for attributes that should only be included in the parent struct, it is
//! clearer if you use the `#[substruct_attr]` attribute macro documented below.
//!
//!
//! # Overriding documentation for emitted structs and fields
//! Sometimes you may want to override the emitted documentation for a struct
//! or field. To do so, document the struct identifier within the `#[substruct]`
//! parameters:
//!
//! ```
//! # use substruct::substruct;
//! #
//! /// All the parameters.
//! #[substruct(
//!     /// A smaller set of parameters.
//!     FilteredParams
//! )]
//! #[derive(Clone, Debug)]
//! pub struct Params {
//!     #[substruct(FilteredParams)]
//!     pub limit: bool,
//!     pub filter: String,
//! }
//! ```
//!
//! For consistency, you can also specify the docs for the parent struct within
//! the `#[substruct]` attribute:
//!
//! ```
//! # use substruct::substruct;
//! #[substruct(
//!     /// The big kahuna.
//!     BigKahuna,
//!
//!     /// The not-so-big kahuna.
//!     SmallKahuna
//! )]
//! #[derive(Clone, Debug)]
//! pub struct BigKahuna {
//!     pub name: String,
//!
//!     #[substruct(SmallKahuna)]
//!     pub profession: String,
//! }
//! ```
//!
//! # Managing attributes on generated structs
//! Sometimes you may want attributes to only apply to some of the emitted
//! structs. To do so, you can use the `#[substruct_attr]` macro to only emit
//! these attributes on the desired structs:
//! ```
//! # use substruct::substruct;
//! # use serde::{Serialize, Deserialize};
//! #
//! #[substruct(ThingB, ThingC, ThingD)]
//! #[derive(Serialize, Deserialize)]
//! pub struct ThingA {
//!     // This field is present in ThingA, ThingB, and ThingC but only has the
//!     // serde rename attribute in ThingB.
//!     #[substruct(ThingB, ThingC)]
//!     #[substruct_attr(ThingB, serde(rename = "a2"))]
//!     pub a: String,
//!
//!     // You can also use the parent struct as a filter.
//!     #[substruct(ThingD, ThingC)]
//!     #[substruct_attr(ThingA, serde(alias = "d"))]
//!     pub b: usize,
//! }
//! ```
//!
//! For more complicated use cases `#[substruct_attr]` supports a similar
//! expression language to the `#[cfg]` macro.
//!
//! ```
//! # use substruct::substruct;
//! # use serde::{Serialize, Deserialize};
//! #
//! #[substruct(A, B, C, D, E, F)]
//! #[derive(Clone, Debug, Serialize, Deserialize)]
//! pub struct A {
//!     // This field is available on all structs except and the serde
//!     // attribute is available on all structs except D (and B).
//!     #[substruct(not(B))]
//!     #[substruct_attr(not(D), serde(alias = "f2"))]
//!     pub f1: u32,
//! }
//! ```
//!
//! The expressions you can use here are
//! - `<ident>` - true when emitting a struct with the same name
//! - `not(<expr>)` - true if the inner expression is false
//! - `any(<expr>...)` - true if _any_ of the inner expressions are true
//! - `all(<expr>...)` - true if _all_ of the inner expressions are true
//!
//! On struct fields, the `#[substruct]` entries are implicitly wrapped in an
//! `any` expression so you can do:
//!
//! ```
//! # use substruct::substruct;
//! #[substruct(A, B, C, D, E, F)]
//! pub struct A {
//!     // This field is available on A, B, C, and D
//!     #[substruct(any(B, C), D)]
//!     pub f1: u32,
//! }
//! ```
//!
//! > The parent struct as always implicitly included in the set of structs
//! > that each field is emitted for. This means that putting `not(A)` in the
//! > the struct above would not exclude the field from `A` (and is, in fact,
//! > equivalent to `all()`).
//!
//! On its own, this isn't too useful, but where it does become useful is when
//! combined with documentation comment overrides.
//!
//! ```
//! # use substruct::substruct;
//! #[substruct(A, B, C, D, E, F)]
//! pub struct A {
//!     /// This is the default documentation.
//!     #[substruct(
//!         /// This is the documentation on C, D, and F
//!         any(C, D, F),
//!
//!         /// And this is the documentation on B
//!         B
//!     )]
//!     pub f1: u32,
//! }
//! ```
//!
//! If multiple documentation overrides apply to a single field, then the first
//! one to apply will be used.
//!
//! # Generics
//! Generics are currently _mostly_ supported. You can use generics with
//! `#[substruct]` and the macro will expand them just fine:
//! ```
//! # use substruct::substruct;
//! #[substruct(SmallGeneric)]
//! pub struct Generic<'a, T> {
//!     pub len: usize,
//!
//!     #[substruct(SmallGeneric)]
//!     pub value: &'a T,
//! }
//! ```
//!
//! However, if one of the child structs doesn't include a field that uses the
//! generic parameter or lifetime then that will result in an error
//! ```compile_fail
//! #[substruct(NoLifetime)]
//! pub struct UsesLifetime<'a> {
//!     //                  ^^ error: lifetime not used in NoLifetime
//!     #[substruct(NoLifetime)]
//!     pub name: String,
//!     pub text: &'a str,
//! }
//! ```

use proc_macro::TokenStream;

#[allow(dead_code)]
#[doc = include_str!("../README.md")]
#[cfg(doc)]
mod readme {}

mod expr;
mod substruct;

/// `#[substruct]` attribute macro.
///
/// See the [crate docs](crate) for detailed docs.
#[proc_macro_attribute]
pub fn substruct(attr: TokenStream, item: TokenStream) -> TokenStream {
    match crate::substruct::expand(attr.into(), item.into()) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
