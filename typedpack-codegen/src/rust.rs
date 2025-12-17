use crate::{Enum, Struct, StructField, StructFieldType};

impl StructFieldType<'_> {
    /// Returns the corresponding Rust type.
    pub fn rust_type(&self) -> String {
        match self {
            Self::Bool => String::from("::std::primitive::bool"),
            Self::U8 => String::from("::std::primitive::u8"),
            Self::I8 => String::from("::std::primitive::i8"),
            Self::U16 => String::from("::std::primitive::u16"),
            Self::I16 => String::from("::std::primitive::i16"),
            Self::U32 => String::from("::std::primitive::u32"),
            Self::I32 => String::from("::std::primitive::i32"),
            Self::U64 => String::from("::std::primitive::u64"),
            Self::I64 => String::from("::std::primitive::i64"),
            Self::F32 => String::from("::std::primitive::f32"),
            Self::F64 => String::from("::std::primitive::f64"),
            Self::String => String::from("::std::string::String"),
            Self::Bytes { len } => {
                if let Some(len) = len {
                    format!("::typedpack::serde_bytes::ByteArray<{len}>")
                } else {
                    String::from("::typedpack::serde_bytes::ByteBuf")
                }
            }
            Self::Array { items } => {
                format!("::std::boxed::Box<[{items}]>", items = items.rust_type())
            }
            Self::Reference { name } => (*name).to_owned(),
        }
    }
}

impl StructField<'_> {
    /// Generates a Rust `struct` field definition.
    pub fn rust_struct_field(&self) -> String {
        let mut s = format!("pub r#{name}: ", name = self.name);
        if self.optional {
            s.push_str("::std::option::Option<");
        }
        if self.nullable {
            s.push_str("::std::option::Option<");
        }
        s.push_str(&self.r#type.rust_type());
        if self.nullable {
            s.push('>');
        }
        if self.optional {
            s.push('>');
        }
        s.push(',');
        s
    }
}

impl Struct<'_> {
    /// Generates a Rust `struct` definition.
    pub fn rust_struct(&self) -> String {
        let mut s = format!(
            "#[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq, ::std::hash::Hash)]\npub struct r#{name} {{\n",
            name = self.name
        );
        for field in &self.fields {
            s.push_str("    ");
            s.push_str(&field.rust_struct_field());
            s.push('\n');
        }
        s.push_str(&format!(
            "}}

impl ::typedpack::serde::Serialize for r#{name} {{
    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: ::typedpack::serde::Serializer,
    {{
        let map_len = ",
            name = self.name
        ));
        let mut required_field_count = 0;
        let mut optional_field_names = Vec::new();
        for field in &self.fields {
            if field.optional {
                optional_field_names.push(&field.name);
            } else {
                required_field_count += 1;
            }
        }
        s.push_str(&format!("{required_field_count}"));
        for optional_field_name in optional_field_names {
            s.push_str(&format!(
                "\n            + if self.r#{name}.is_some() {{ 1 }} else {{ 0 }}",
                name = optional_field_name
            ));
        }
        s.push_str(";\n        let ");
        if !self.fields.is_empty() {
            s.push_str("mut ");
        }
        s.push_str("map = serializer.serialize_map(::std::option::Option::Some(map_len))?;\n");
        for field in &self.fields {
            if field.optional {
                s.push_str(&format!(
                    "        if self.r#{name}.is_some() {{\n    ",
                    name = field.name
                ));
            }
            s.push_str(&format!("        ::typedpack::serde::ser::SerializeMap::serialize_entry(&mut map, &{id}, &self.r#{name})?;\n", id = field.id, name = field.name));
            if field.optional {
                s.push_str("        }\n");
            }
        }
        s.push_str("        ::typedpack::serde::ser::SerializeMap::end(map)\n    }\n}\n\n");

        s.push_str("\nimpl<'de> ::typedpack::serde::Deserialize<'de> for r#");
        s.push_str(self.name);
        s.push_str(" {\nfn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>\n    where\n        D: ::typedpack::serde::Deserializer<'de>,\n    {\n        ");

        if !self.fields.is_empty() {
            s.push_str("let entries = ");
        }
        s.push_str("deserializer.deserialize_map(::typedpack::MapVisitor)?;\n");

        if self.fields.is_empty() {
            s.push_str("        ::std::result::Result::Ok(Self {})\n    }\n}");
            return s;
        }

        for field in &self.fields {
            s.push_str("        let mut r#");
            s.push_str(field.name);
            s.push_str(" = ::std::option::Option::None;\n");
        }
        s.push_str("\n        for (key, value) in entries {\n            match key {\n");

        for field in &self.fields {
            s.push_str(&format!("                {id} => {{
                    r#{name} = ::std::option::Option::Some(::typedpack::FromRmpValue::from(value).map_err(|err| ::typedpack::serde::de::Error::custom(err))?);
                }}
", id = field.id, name = field.name));
        }
        s.push_str("                _ => {{}}\n            }\n        }\n        ::std::result::Result::Ok(Self {\n");
        for field in &self.fields {
            s.push_str("            r#");
            s.push_str(field.name);
            if field.optional {
                s.push_str(",\n");
            } else {
                s.push_str(": r#");
                s.push_str(field.name);
                s.push_str(".ok_or_else(|| ::typedpack::serde::de::Error::custom(\"missing `");
                s.push_str(field.name);
                s.push_str("`\"))?,\n");
            }
        }
        s.push_str("        })\n    }\n}\n\n");

        s.push_str("impl ::typedpack::FromRmpValue for r#");
        s.push_str(self.name);
        s.push_str("{\n    fn from(value: ::typedpack::rmpv::Value) -> ::std::result::Result<Self, &'static ::std::primitive::str> {\n        ::typedpack::serde::Deserialize::deserialize(value).map_err(|_| \"could not deserialize struct\")\n    }\n}");
        s
    }
}

impl Enum<'_> {
    /// Generates a Rust `enum` definition.
    pub fn rust_enum(&self) -> String {
        match self {
            Self::Tagged { name, variants } => {
                let mut s = String::from(
                    "#[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq, ::std::hash::Hash)]\npub enum r#",
                );
                s.push_str(name);
                s.push_str(" {\n");

                for variant in variants {
                    s.push_str("    r#");
                    s.push_str(variant.name);
                    s.push_str("(r#");
                    s.push_str(variant.r#type);
                    s.push_str("),\n");
                }
                s.push_str("}\n\nimpl ::typedpack::serde::Serialize for r#");
                s.push_str(name);
                s.push_str(" {\n    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>\n    where\n        S: ::typedpack::serde::Serializer,\n    {\n        let mut seq = serializer.serialize_seq(::std::option::Option::Some(2))?;\n        match self {\n");

                for variant in variants {
                    s.push_str("            Self::r#");
                    s.push_str(variant.name);
                    s.push_str("(data) => {\n                ::typedpack::serde::ser::SerializeSeq::serialize_element(&mut seq, &");
                    s.push_str(&variant.id.to_string());
                    s.push_str(")?;\n                ::typedpack::serde::ser::SerializeSeq::serialize_element(&mut seq, data)?;\n            }\n");
                }
                s.push_str("        }\n        ::typedpack::serde::ser::SerializeSeq::end(seq)\n    }\n}\n\nimpl<'de> ::typedpack::serde::Deserialize<'de> for r#");
                s.push_str(name);
                s.push_str(" {\n    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>\n    where\n        D: ::typedpack::serde::Deserializer<'de>,\n    {\n        struct Visitor;\n        impl<'de> ::typedpack::serde::de::Visitor<'de> for Visitor {\n            type Value = r#");
                s.push_str(name);
                s.push_str(";\n\n            fn expecting(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {\n                formatter.write_str(\"a 2-tuple\")\n            }\n\n            fn visit_seq<A>(self, mut seq: A) -> ::std::result::Result<Self::Value, A::Error>\n             where\n                A: ::typedpack::serde::de::SeqAccess<'de>,\n            {\n                let ::std::option::Option::Some(id) = seq.next_element()? else {\n                    return ::std::result::Result::Err(::typedpack::serde::de::Error::custom(\"invalid enum tag\"));\n                };\n                let value = match id {\n");

                for variant in variants {
                    s.push_str("                    ");
                    s.push_str(&variant.id.to_string());
                    s.push_str("u8 => r#");
                    s.push_str(name);
                    s.push_str("::r#");
                    s.push_str(variant.name);
                    s.push_str("(seq.next_element()?.ok_or_else(|| ::typedpack::serde::de::Error::custom(\"invalid tagged enum data\"))?),\n");
                }

                s.push_str("                    _ => {\n                        return ::std::result::Result::Err(::typedpack::serde::de::Error::custom(\"invalid enum tag\"));\n                    }\n                };\n                if seq.next_element::<::typedpack::serde::de::IgnoredAny>()?.is_some() {\n                    return ::std::result::Result::Err(::typedpack::serde::de::Error::custom(\"invalid tagged enum data\"));\n                }\n                ::std::result::Result::Ok(value)\n            }\n        }\n\n        deserializer.deserialize_tuple(2, Visitor)\n    }\n}");
                s
            }
            Self::Untagged { name, variants } => {
                let mut s = String::from(
                    "#[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq, ::std::hash::Hash, ::typedpack::serde_repr::Serialize_repr, ::typedpack::serde_repr::Deserialize_repr)]\n#[repr(u8)]\npub enum r#",
                );
                s.push_str(name);
                s.push_str(" {\n");

                for variant in variants {
                    s.push_str("    r#");
                    s.push_str(variant.name);
                    s.push_str(" = ");
                    s.push_str(&variant.id.to_string());
                    s.push_str(",\n");
                }
                s.push('}');
                s
            }
        }
    }
}
