use std::rc::Rc;

mod parse;
mod rust;
pub mod typescript;

pub use parse::{ParseError, parse};

/// The type of a typedpack `struct` field.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StructFieldType<'a> {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    String,
    Bytes { len: Option<u32> },
    Array { items: Rc<StructFieldType<'a>> },
    Reference { name: &'a str },
}

/// A typedpack `struct` field.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructField<'a> {
    id: u8,
    name: &'a str,
    r#type: StructFieldType<'a>,
    optional: bool,
    nullable: bool,
}

/// A typedpack `struct`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Struct<'a> {
    name: &'a str,
    fields: Vec<StructField<'a>>,
}

/// A variant of a typedpack tagged `enum`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaggedEnumVariant<'a> {
    id: u8,
    name: &'a str,
    r#type: &'a str,
}

/// A variant of a typedpack untagged `enum`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UntaggedEnumVariant<'a> {
    id: u8,
    name: &'a str,
}

/// A typedpack `enum`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Enum<'a> {
    Tagged {
        name: &'a str,
        variants: Vec<TaggedEnumVariant<'a>>,
    },
    Untagged {
        name: &'a str,
        variants: Vec<UntaggedEnumVariant<'a>>,
    },
}

/// A typedpack `enum` or `struct`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type<'a> {
    Enum(Enum<'a>),
    Struct(Struct<'a>),
}

impl Type<'_> {
    /// Returns the name of the type.
    pub fn name(&self) -> &str {
        match self {
            Self::Enum(Enum::Tagged { name, .. }) | Self::Enum(Enum::Untagged { name, .. }) => name,
            Self::Struct(r#struct) => r#struct.name,
        }
    }
}
