# substruct

Substruct is a proc-macro wich allows you to easily declare strucs which are
subsets of another struct.

## Simple Example
A basic use of substruct looks like this

```rust
use substruct::substruct;

#[substruct(LimitedQueryParams)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryParams {
    #[substruct(LimitedQueryParams)]
    pub name: Option<String>,

    #[substruct(LimitedQueryParams)]
    pub parent: Option<String>,

    pub limit: usize
}
```

which expands out to produce
```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryParams {
    pub name: Option<String>,
    pub parent: Option<String>,
    pub limit: usize
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LimitedQueryParams {
    pub name: Option<String>,
    pub parent: Option<String>,
}
```

## Complex Example
Substruct also supports copying attributes or adding attributes specific to a
subset of the child structs.

```rust
use std::time::SystemTime;
use substruct::substruct;

#[substruct(PostQueryParams, ThreadQueryParams)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct QueryParams {
    /// Query only within forums with this id
    pub forum: Option<u64>,

    /// Query only with threads with this id.
    #[substruct(PostQueryParams)]
    pub thread: Option<u64>,

    /// The username to search.
    #[substruct(PostQueryParams, ThreadQueryParams)]
    // Alias only applied for ThreadQueryParams
    #[substruct_attr(ThreadQueryParams, serde(alias = "username"))]
    pub user: Option<String>,

    #[substruct(PostQueryParams, ThreadQueryParams)]
    // Field is renamed (in serde) for PostQueryParams and ThreadQueryParams
    // but not for QueryParams.
    #[substruct_attr(not(QueryParams), serde(rename = "before_ts"))]
    pub before: Option<SystemTime>,

    #[substruct(PostQueryParams, ThreadQueryParams)]
    #[substruct_attr(not(QueryParams), serde(rename = "before_ts"))]
    pub after: Option<SystemTime>,

    // Limit is only present on QueryParams.
    pub limit: Option<usize>,
}
```

## Limitations
Substruct supports generics but will fail if the generic parameters are not
used by all of the child structs.


# See Also
- The [subenum](https://crates.io/crates/subenum)