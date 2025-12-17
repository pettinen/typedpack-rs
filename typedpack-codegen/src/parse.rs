use std::collections::HashSet;

use crate::{
    Enum, Struct, StructField, StructFieldType, TaggedEnumVariant, Type, UntaggedEnumVariant,
};

/// The error type which is returned from [`parse`].
#[derive(Clone, Debug, thiserror::Error)]
pub enum ParseError {
    #[error("line {line}, column {column}: duplicate enum variant ID '{id}'", line = pos.line, column = pos.column)]
    DuplicateEnumVariantId { id: u8, pos: Position },
    #[error("line {line}, column {column}: duplicate enum variant name '{name}'", line = pos.line, column = pos.column)]
    DuplicateEnumVariantName { name: String, pos: Position },
    #[error("line {line}, column {column}: duplicate struct field ID '{id}'", line = pos.line, column = pos.column)]
    DuplicateStructFieldId { id: u8, pos: Position },
    #[error("line {line}, column {column}: duplicate struct field name '{name}'", line = pos.line, column = pos.column)]
    DuplicateStructFieldName { name: String, pos: Position },
    #[error("line {line}, column {column}: duplicate type name '{name}'", line = pos.line, column = pos.column)]
    DuplicateTypeName { name: String, pos: Position },
    #[error("line {line}, column {column}: invalid enum variant ID '{id}'\nnote: ID must be between 0 and 127 inclusive and leading zeros are not allowed", line = pos.line, column = pos.column)]
    InvalidEnumVariantId { id: String, pos: Position },
    #[error("line {line}, column {column}: invalid struct field ID '{id}'\nnote: ID must be between 0 and 127 inclusive and leading zeros are not allowed", line = pos.line, column = pos.column)]
    InvalidStructFieldId { id: String, pos: Position },
    #[error("line {line}, column {column}: cannot mix tagged and untagged enum variants", line = pos.line, column = pos.column)]
    MixedTaggedAndUntaggedEnumVariants { pos: Position },
    #[error("line {line}, column {column}: unexpected non-ASCII character '{char}'", line = pos.line, column = pos.column)]
    NonAsciiCharacter { char: char, pos: Position },
    #[error("unexpected end of input: expected {expected}")]
    UnexpectedEnd { expected: &'static str },
    #[error("line {line}, column {column}: unexpected token '{unexpected}'; expected {expected}", line = pos.line, column = pos.column)]
    UnexpectedToken {
        unexpected: String,
        expected: &'static str,
        pos: Position,
    },
    #[error("unknown tagged enum variant type '{0}'")]
    UnknownTaggedEnumVariantType(String),
}

/// The location a [`ParseError`] occurred at in the source file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    line: usize,
    column: usize,
}

#[derive(Clone, Copy, Debug)]
struct Token<'a> {
    str: &'a str,
    pos: Position,
}

fn tokenize(s: &str) -> Result<Vec<Token<'_>>, ParseError> {
    let mut tokens = Vec::new();

    let mut current_token_start_idx_and_column = None;
    let mut inside_comment = false;
    let mut line = 1;
    let mut column = 0;
    for (i, char) in s.char_indices() {
        if inside_comment && char != '\n' {
            column += 1;
            continue;
        }
        if char.is_ascii_alphanumeric() || char == '_' {
            column += 1;
            if current_token_start_idx_and_column.is_none() {
                current_token_start_idx_and_column = Some((i, column));
            }
        } else {
            column += 1;
            if let Some((idx, column)) = current_token_start_idx_and_column {
                tokens.push(Token {
                    str: &s[idx..i],
                    pos: Position { line, column },
                });
                current_token_start_idx_and_column = None;
            }
            if !char.is_ascii() {
                return Err(ParseError::NonAsciiCharacter {
                    char,
                    pos: Position { line, column },
                });
            }
            if char == '#' {
                inside_comment = true;
            } else if char == '\n' {
                line += 1;
                column = 0;
                inside_comment = false;
            } else if !char.is_whitespace() {
                tokens.push(Token {
                    str: &s[i..i + 1],
                    pos: Position { line, column },
                });
            }
        }
    }

    if let Some((start, column)) = current_token_start_idx_and_column {
        tokens.push(Token {
            str: &s[start..],
            pos: Position { line, column },
        });
    }

    Ok(tokens)
}

fn parse_struct_field<'a>(
    tokens: &mut &[Token<'a>],
    field_ids: &HashSet<u8>,
    field_names: &HashSet<&str>,
) -> Result<StructField<'a>, ParseError> {
    let mut optional = false;
    let mut nullable = false;

    if tokens.is_empty() {
        return Err(ParseError::UnexpectedEnd {
            expected: "`optional`, `nullable` or a struct field type",
        });
    }

    if tokens[0].str == "optional" {
        optional = true;
        tokens.split_off_first();

        if tokens.is_empty() {
            return Err(ParseError::UnexpectedEnd {
                expected: "`nullable` or a struct field type",
            });
        }
    }

    if tokens[0].str == "nullable" {
        nullable = true;
        tokens.split_off_first();
    }

    let Some(type_name) = tokens.split_off_first() else {
        return Err(ParseError::UnexpectedEnd {
            expected: "a struct field type",
        });
    };
    if !type_name
        .str
        .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
    {
        return Err(ParseError::UnexpectedToken {
            unexpected: type_name.str.into(),
            expected: if nullable {
                "a struct field type"
            } else if optional {
                "`nullable` or a struct field type"
            } else {
                "`optional`, `nullable` or a struct field type"
            },
            pos: type_name.pos,
        });
    }

    let mut array_depth = 0;
    let mut open_array_bracket = false;

    let name = loop {
        match tokens.split_off_first() {
            Some(Token { str: "[", pos }) => {
                if open_array_bracket {
                    return Err(ParseError::UnexpectedToken {
                        unexpected: "[".into(),
                        expected: "`]`",
                        pos: *pos,
                    });
                }
                array_depth += 1;
                open_array_bracket = true;
            }
            Some(Token { str: "]", pos }) => {
                if !open_array_bracket {
                    return Err(ParseError::UnexpectedToken {
                        unexpected: "]".into(),
                        expected: "`[` or a struct field name",
                        pos: *pos,
                    });
                }
                open_array_bracket = false;
            }
            Some(field_name) => {
                if open_array_bracket {
                    return Err(ParseError::UnexpectedToken {
                        unexpected: field_name.str.into(),
                        expected: "`]`",
                        pos: field_name.pos,
                    });
                }
                break field_name;
            }
            None => {
                return Err(ParseError::UnexpectedEnd {
                    expected: "`[` or a struct field name",
                });
            }
        }
    };
    if !name
        .str
        .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
    {
        return Err(ParseError::UnexpectedToken {
            unexpected: name.str.into(),
            expected: "a struct field name",
            pos: name.pos,
        });
    }
    if field_names.contains(&name.str) {
        return Err(ParseError::DuplicateStructFieldName {
            name: name.str.into(),
            pos: name.pos,
        });
    }

    match tokens.split_off_first() {
        Some(Token { str: "=", .. }) => {}
        Some(token) => {
            return Err(ParseError::UnexpectedToken {
                unexpected: token.str.into(),
                expected: "`=`",
                pos: token.pos,
            });
        }
        None => {
            return Err(ParseError::UnexpectedEnd { expected: "`=`" });
        }
    }

    let Some(id) = tokens.split_off_first() else {
        return Err(ParseError::UnexpectedEnd {
            expected: "an integer between 0 and 127 inclusive",
        });
    };
    if !id.str.chars().all(|c| c.is_ascii_digit()) {
        return Err(ParseError::UnexpectedToken {
            unexpected: id.str.into(),
            expected: "an integer between 0 and 127 inclusive",
            pos: id.pos,
        });
    }
    if id.str.starts_with('0') && id.str != "0" {
        return Err(ParseError::InvalidStructFieldId {
            id: id.str.into(),
            pos: id.pos,
        });
    }
    let id_pos = id.pos;
    let id = match id.str.parse() {
        Ok(id @ ..128) => id,
        Ok(_) | Err(_) => {
            return Err(ParseError::InvalidStructFieldId {
                id: id.str.into(),
                pos: id.pos,
            });
        }
    };
    if field_ids.contains(&id) {
        return Err(ParseError::DuplicateStructFieldId { id, pos: id_pos });
    }

    if tokens.is_empty() {
        return Err(ParseError::UnexpectedEnd { expected: ";" });
    }

    match tokens.split_off_first() {
        Some(Token { str: ";", .. }) => {}
        Some(token) => {
            return Err(ParseError::UnexpectedToken {
                unexpected: token.str.into(),
                expected: "`;`",
                pos: token.pos,
            });
        }
        None => {
            return Err(ParseError::UnexpectedEnd { expected: "`;`" });
        }
    }

    let bytes_type = {
        if type_name.str == "bytes" {
            Some(StructFieldType::Bytes { len: None })
        } else if type_name.str == "bytes0" {
            Some(StructFieldType::Bytes { len: Some(0) })
        } else if let Some(len_str) = type_name.str.strip_prefix("bytes") {
            match len_str.chars().next() {
                Some('1'..='9') => match len_str.parse() {
                    Ok(len) => Some(StructFieldType::Bytes { len: Some(len) }),
                    Err(_) => None,
                },
                _ => None,
            }
        } else {
            None
        }
    };
    let mut r#type = if let Some(bytes_type) = bytes_type {
        bytes_type
    } else {
        match type_name.str {
            "bool" => StructFieldType::Bool,
            "uint8" => StructFieldType::U8,
            "int8" => StructFieldType::I8,
            "uint16" => StructFieldType::U16,
            "int16" => StructFieldType::I16,
            "uint32" => StructFieldType::U32,
            "int32" => StructFieldType::I32,
            "uint64" => StructFieldType::U64,
            "int64" => StructFieldType::I64,
            "float32" => StructFieldType::F32,
            "float64" => StructFieldType::F64,
            "string" => StructFieldType::String,
            name => StructFieldType::Reference { name },
        }
    };

    for _ in 0..array_depth {
        r#type = StructFieldType::Array {
            items: r#type.into(),
        };
    }
    Ok(StructField {
        id,
        name: name.str,
        r#type,
        optional,
        nullable,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum EnumVariant<'a> {
    Tagged(TaggedEnumVariant<'a>),
    Untagged(UntaggedEnumVariant<'a>),
}

fn parse_enum_variant<'a>(
    tokens: &mut &[Token<'a>],
    variant_ids: &HashSet<u8>,
    variant_names: &HashSet<&str>,
    is_tagged: Option<bool>,
) -> Result<EnumVariant<'a>, ParseError> {
    let Some(type_or_name) = tokens.split_off_first() else {
        return Err(ParseError::UnexpectedEnd {
            expected: match is_tagged {
                Some(true) => "a struct name",
                Some(false) => "an enum variant name",
                None => "a struct name or an enum variant name",
            },
        });
    };

    if !type_or_name
        .str
        .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
    {
        return Err(ParseError::UnexpectedToken {
            unexpected: type_or_name.str.into(),
            expected: match is_tagged {
                Some(true) => "a struct name",
                Some(false) => "an enum variant name",
                None => "a struct name or an enum variant name",
            },
            pos: type_or_name.pos,
        });
    }

    let (r#type, name) = match tokens.split_off_first() {
        Some(Token { str: "=", pos }) => {
            if is_tagged == Some(true) {
                return Err(ParseError::UnexpectedToken {
                    unexpected: "=".into(),
                    expected: "an enum variant name",
                    pos: *pos,
                });
            }
            (None, type_or_name)
        }
        Some(token) => {
            if !token
                .str
                .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            {
                return Err(ParseError::UnexpectedToken {
                    unexpected: token.str.into(),
                    expected: match is_tagged {
                        Some(true) => "an enum variant name",
                        Some(false) => "`=`",
                        None => "an enum variant name or `=`",
                    },
                    pos: token.pos,
                });
            }
            (Some(type_or_name), token)
        }
        None => {
            return Err(ParseError::UnexpectedEnd {
                expected: match is_tagged {
                    Some(true) => "an enum variant name",
                    Some(false) => "`=`",
                    None => "an enum variant name or `=`",
                },
            });
        }
    };

    if r#type.is_some() {
        match tokens.split_off_first() {
            Some(Token { str: "=", .. }) => {}
            Some(token) => {
                return Err(ParseError::UnexpectedToken {
                    unexpected: token.str.into(),
                    expected: "`=`",
                    pos: token.pos,
                });
            }
            None => {
                return Err(ParseError::UnexpectedEnd { expected: "`=`" });
            }
        }
    }

    if variant_names.contains(&name.str) {
        return Err(ParseError::DuplicateEnumVariantName {
            name: name.str.into(),
            pos: name.pos,
        });
    }

    let Some(id) = tokens.split_off_first() else {
        return Err(ParseError::UnexpectedEnd {
            expected: "an integer between 0 and 127 inclusive",
        });
    };
    if !id.str.chars().all(|c| c.is_ascii_digit()) {
        return Err(ParseError::UnexpectedToken {
            unexpected: id.str.into(),
            expected: "an integer between 0 and 127 inclusive",
            pos: id.pos,
        });
    }
    if id.str.starts_with('0') && id.str != "0" {
        return Err(ParseError::InvalidEnumVariantId {
            id: id.str.into(),
            pos: id.pos,
        });
    }
    let id_pos = id.pos;
    let id = match id.str.parse() {
        Ok(id @ ..128) => id,
        Ok(_) | Err(_) => {
            return Err(ParseError::InvalidEnumVariantId {
                id: id.str.into(),
                pos: id.pos,
            });
        }
    };
    if variant_ids.contains(&id) {
        return Err(ParseError::DuplicateEnumVariantId { id, pos: id_pos });
    }

    match tokens.split_off_first() {
        Some(Token { str: ";", .. }) => {}
        Some(token) => {
            return Err(ParseError::UnexpectedToken {
                unexpected: token.str.into(),
                expected: "`;`",
                pos: token.pos,
            });
        }
        None => {
            return Err(ParseError::UnexpectedEnd { expected: "`;`" });
        }
    }

    Ok(match r#type {
        Some(r#type) => EnumVariant::Tagged(TaggedEnumVariant {
            id,
            name: name.str,
            r#type: r#type.str,
        }),
        None => EnumVariant::Untagged(UntaggedEnumVariant { id, name: name.str }),
    })
}

fn parse_enum<'a>(
    tokens: &mut &[Token<'a>],
    type_names: &HashSet<&str>,
) -> Result<Enum<'a>, ParseError> {
    match tokens.split_off_first() {
        Some(Token { str: "enum", .. }) => {}
        Some(token) => {
            return Err(ParseError::UnexpectedToken {
                unexpected: token.str.into(),
                expected: "`enum`",
                pos: token.pos,
            });
        }
        None => {
            return Err(ParseError::UnexpectedEnd { expected: "`enum`" });
        }
    }

    let Some(name) = tokens.split_off_first() else {
        return Err(ParseError::UnexpectedEnd {
            expected: "a type name",
        });
    };
    if !name
        .str
        .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
    {
        return Err(ParseError::UnexpectedToken {
            unexpected: name.str.into(),
            expected: "a type name",
            pos: name.pos,
        });
    }
    if type_names.contains(&name.str) {
        return Err(ParseError::DuplicateTypeName {
            name: name.str.into(),
            pos: name.pos,
        });
    }

    match tokens.split_off_first() {
        Some(Token { str: "{", .. }) => {}
        Some(token) => {
            return Err(ParseError::UnexpectedToken {
                unexpected: token.str.into(),
                expected: "`{`",
                pos: token.pos,
            });
        }
        None => {
            return Err(ParseError::UnexpectedEnd { expected: "`{`" });
        }
    }

    let mut variants = Vec::new();
    let mut variant_ids = HashSet::new();
    let mut variant_names = HashSet::new();
    let mut is_tagged = None;

    while !tokens.is_empty() && tokens[0].str != "}" {
        let pos = tokens[0].pos;
        let variant = parse_enum_variant(tokens, &variant_ids, &variant_names, is_tagged)?;
        if is_tagged.is_none() {
            is_tagged = Some(matches!(variant, EnumVariant::Tagged(_)));
        } else if is_tagged != Some(matches!(variant, EnumVariant::Tagged(_))) {
            return Err(ParseError::MixedTaggedAndUntaggedEnumVariants { pos });
        }
        match variant {
            EnumVariant::Tagged(TaggedEnumVariant { id, name, .. })
            | EnumVariant::Untagged(UntaggedEnumVariant { id, name }) => {
                variant_ids.insert(id);
                variant_names.insert(name);
            }
        }
        variants.push(variant);
    }

    match tokens.split_off_first() {
        Some(Token { str: "}", pos }) => {
            if variants.is_empty() {
                return Err(ParseError::UnexpectedToken {
                    unexpected: '}'.into(),
                    expected: "an enum variant",
                    pos: *pos,
                });
            }
        }
        Some(token) => {
            return Err(ParseError::UnexpectedToken {
                unexpected: token.str.into(),
                expected: if variants.is_empty() {
                    "an enum variant"
                } else {
                    "an enum variant or `}`"
                },
                pos: token.pos,
            });
        }
        None => {
            return Err(ParseError::UnexpectedEnd {
                expected: "an enum variant or `}`",
            });
        }
    }

    Ok(match variants[0] {
        EnumVariant::Tagged(_) => Enum::Tagged {
            name: name.str,
            variants: variants
                .into_iter()
                .map(|variant| match variant {
                    EnumVariant::Tagged(variant) => variant,
                    EnumVariant::Untagged(_) => unreachable!(),
                })
                .collect(),
        },
        EnumVariant::Untagged(_) => Enum::Untagged {
            name: name.str,
            variants: variants
                .into_iter()
                .map(|variant| match variant {
                    EnumVariant::Tagged(_) => unreachable!(),
                    EnumVariant::Untagged(variant) => variant,
                })
                .collect(),
        },
    })
}

fn parse_struct<'a>(
    tokens: &mut &[Token<'a>],
    type_names: &HashSet<&str>,
) -> Result<Struct<'a>, ParseError> {
    match tokens.split_off_first() {
        Some(Token { str: "struct", .. }) => {}
        Some(token) => {
            return Err(ParseError::UnexpectedToken {
                unexpected: token.str.into(),
                expected: "`struct`",
                pos: token.pos,
            });
        }
        None => {
            return Err(ParseError::UnexpectedEnd {
                expected: "`struct`",
            });
        }
    }
    let Some(name) = tokens.split_off_first() else {
        return Err(ParseError::UnexpectedEnd {
            expected: "a type name",
        });
    };
    if !name
        .str
        .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
    {
        return Err(ParseError::UnexpectedToken {
            unexpected: name.str.into(),
            expected: "a type name",
            pos: name.pos,
        });
    }
    if type_names.contains(&name.str) {
        return Err(ParseError::DuplicateTypeName {
            name: name.str.into(),
            pos: name.pos,
        });
    }

    match tokens.split_off_first() {
        Some(Token { str: "{", .. }) => {}
        Some(token) => {
            return Err(ParseError::UnexpectedToken {
                unexpected: token.str.into(),
                expected: "`{`",
                pos: token.pos,
            });
        }
        None => {
            return Err(ParseError::UnexpectedEnd { expected: "`{`" });
        }
    }

    let mut fields = Vec::new();
    let mut field_ids = HashSet::new();
    let mut field_names = HashSet::new();

    while !tokens.is_empty() && tokens[0].str != "}" {
        let field = parse_struct_field(tokens, &field_ids, &field_names)?;
        field_ids.insert(field.id);
        field_names.insert(field.name);
        fields.push(field);
    }

    match tokens.split_off_first() {
        Some(Token { str: "}", .. }) => {}
        Some(token) => {
            return Err(ParseError::UnexpectedToken {
                unexpected: token.str.into(),
                expected: "a struct field or `}`",
                pos: token.pos,
            });
        }
        None => {
            return Err(ParseError::UnexpectedEnd {
                expected: "a struct field or `}`",
            });
        }
    }

    Ok(Struct {
        name: name.str,
        fields,
    })
}

fn parse_type<'a>(
    tokens: &mut &[Token<'a>],
    type_names: &HashSet<&str>,
) -> Result<Type<'a>, ParseError> {
    if tokens.is_empty() {
        return Err(ParseError::UnexpectedEnd {
            expected: "`enum` or `struct`",
        });
    }
    if tokens[0].str == "enum" {
        parse_enum(tokens, type_names).map(Type::Enum)
    } else if tokens[0].str == "struct" {
        parse_struct(tokens, type_names).map(Type::Struct)
    } else {
        Err(ParseError::UnexpectedToken {
            unexpected: tokens[0].str.into(),
            expected: "`enum` or `struct`",
            pos: tokens[0].pos,
        })
    }
}

/// Parses a typedpack `.tp` file.
pub fn parse(s: &str) -> Result<Vec<Type<'_>>, ParseError> {
    let tokens = tokenize(s)?;
    let mut tokens = tokens.as_slice();

    let mut types = Vec::new();
    let mut type_names = HashSet::new();
    let mut struct_names = HashSet::new();

    while !tokens.is_empty() {
        let r#type = parse_type(&mut tokens, &type_names)?;
        match &r#type {
            Type::Enum(Enum::Tagged { name, .. }) | Type::Enum(Enum::Untagged { name, .. }) => {
                type_names.insert(name);
            }
            Type::Struct(r#struct) => {
                type_names.insert(r#struct.name);
                struct_names.insert(r#struct.name);
            }
        }
        types.push(r#type);
    }

    // check that each tagged enum variant's type is present
    for r#type in &types {
        if let Type::Enum(Enum::Tagged { variants, .. }) = r#type {
            for variant in variants {
                if !struct_names.contains(variant.r#type) {
                    return Err(ParseError::UnknownTaggedEnumVariantType(
                        variant.r#type.to_owned(),
                    ));
                }
            }
        }
    }

    Ok(types)
}
