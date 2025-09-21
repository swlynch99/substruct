# substruct

![[crates.io](https://crates.io/crate/substruct)](https://img.shields.io/crates/v/substruct)
![[license](https://crates.io/crate/substruct)](https://img.shields.io/crates/l/substruct)
![[docs.rs](https://docs.rs/substruct)](https://img.shields.io/docsrs/substruct)
![ci](https://img.shields.io/github/actions/workflow/status/swlynch99/substruct/cargo.yml)

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

    // This field has a pub(crate) visibility on ThreadQueryParams
    #[substruct(pub(crate) ThreadQueryParams)]
    pub hidden: bool,

    // Limit is only present on QueryParams.
    pub limit: Option<usize>,
}
```

## Field Type Transformations

Sometimes you may want a substruct to have a different type for a field than
the parent struct. For example, you might want an optional field in the parent
to be required in the substruct. This can be achieved by specifying transformations
inline with the struct name:

```rust
use substruct::substruct;

#[substruct(RequiredParams)]
#[derive(Clone, Debug)]
pub struct OptionalParams {
    #[substruct(RequiredParams(unwrap))]
    pub name: Option<String>,

    #[substruct(RequiredParams)]
    pub limit: usize,

    pub description: Option<String>,
}
```

This generates:

```rust,ignore
#[derive(Clone, Debug)]
pub struct RequiredParams {
    pub name: String,  // Note: no longer Option<String>
    pub limit: usize,
}

// Error type for failed conversions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionalParamsConversionError {
    MissingRequiredField(&'static str),
}

// TryFrom implementation instead of From
impl TryFrom<OptionalParams> for RequiredParams {
    type Error = OptionalParamsConversionError;

    fn try_from(value: OptionalParams) -> Result<Self, Self::Error> {
        // Implementation that unwraps Option fields
    }
}

// Method to convert back to parent struct
impl RequiredParams {
    pub fn into_optional_params(self, description: Option<String>) -> OptionalParams {
        // Implementation
    }
}
```

Another example using `try_into` to convert between numeric types:

```rust
use substruct::substruct;

#[substruct(ConvertedParams)]
#[derive(Clone, Debug)]
pub struct OriginalParams {
    #[substruct(ConvertedParams(try_into = u64))]
    pub id: u32,

    #[substruct(ConvertedParams)]
    pub limit: usize,

    pub description: Option<String>,
}
```

This generates:

```rust,ignore
#[derive(Clone, Debug)]
pub struct ConvertedParams {
    pub id: u64,  // Note: converted from u32 to u64
    pub limit: usize,
}

// TryFrom implementation for type conversion
impl TryFrom<OriginalParams> for ConvertedParams {
    type Error = OriginalParamsConversionError;

    fn try_from(value: OriginalParams) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.try_into().map_err(|_|
                OriginalParamsConversionError::ConversionFailed("id"))?,
            limit: value.limit,
        })
    }
}

// Method to convert back using reverse conversion
impl ConvertedParams {
    pub fn into_original_params(self, description: Option<String>) -> OriginalParams {
        OriginalParams {
            id: self.id.try_into().expect("reverse conversion should not fail"),
            limit: self.limit,
            description,
        }
    }
}
```

When field transformations are used:

- A `TryFrom<ParentStruct>` implementation is generated instead of `From`
- A conversion error type `{ParentStruct}ConversionError` is created with variants for different failure types
- An `into_{parent_struct}` method converts back to the parent

Supported transformations are specified inline with the struct name:

- `StructName(unwrap)`: Transforms `Option<T>` to `T` in the substruct
- `StructName(try_into = TargetType)`: Transforms the field type using `TryInto<TargetType>`

## Limitations

Substruct supports generics but will fail if the generic parameters are not
used by all of the child structs.

# See Also

- The [subenum](https://crates.io/crates/subenum) crate offers the same thing
  for enums.
