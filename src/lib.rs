//! Substruct is a simple proc-macro to derive subsets of structs. It allows
//! conversion between the parent and the child, will derive any traits on the
//! child that you have on the parent, along with any attributes as well.
//!
//! # Limitations
//! Currently, substruct will copy all generic parameters on the struct to any
//! of its children. If the child does not use a generic parameter from the
//! parent then this will result in an error.
//!
//! # Examples
//! A simple example:
//! ```
//! use substruct::substruct;
//!
//! #[substruct(SubQuery)]
//! #[derive(Clone, Debug)]
//! pub struct Query {
//!     #[substruct(SubQuery)]
//!     pub a: &'static str,
//!     pub b: usize
//! }
//!
//! let subquery = SubQuery { a: "test query" };
//! let query = subquery.into_query(5);
//!
//! assert_eq!(query.a, "test query");
//! ```

use proc_macro::TokenStream;

mod substruct;

/// `#[substruct]` attribute macro.
///
/// See the [crate docs](crate) for detailed docs.
#[proc_macro_attribute]
pub fn substruct(attr: TokenStream, mut item: TokenStream) -> TokenStream {
    match crate::substruct::expand(attr.into(), item.clone().into()) {
        Ok(tokens) => tokens.into(),
        Err(e) => {
            let tokens: TokenStream = e.to_compile_error().into();
            item.extend(tokens);
            item
        }
    }
}
