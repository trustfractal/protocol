use blake2::{Blake2b, Digest};
use core::convert::TryInto;

use crate::{Error, Object, Value};

pub type Id = [u8; 8];

pub struct StructDef {
    pub(crate) type_name: String,
    pub(crate) fields: Vec<FieldDef>,
}

impl StructDef {
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn id(&self) -> [u8; 8] {
        let mut hasher = Blake2b::new();

        hasher.update(&self.type_name);
        for field in &self.fields {
            hasher.update(&field.name);
            hasher.update(field.type_.id());
        }

        let digest = hasher.finalize();

        let bytes: &[u8] = &digest[0..8];
        bytes.try_into().expect("hash should always be > 8 bytes")
    }

    pub fn fields(&self) -> &[FieldDef] {
        self.fields.as_ref()
    }

    pub fn parse<'i>(&self, mut bytes: &'i [u8]) -> Result<Object, Error<'i>> {
        let mut values = Vec::with_capacity(self.fields.len());

        for field in &self.fields {
            let (new_bytes, value) = field.parse(bytes)?;
            bytes = new_bytes;
            values.push(value);
        }
        if bytes.len() > 0 {
            return Err(Error::TooManyBytes);
        }

        Ok(Object {
            schema: self,
            values,
        })
    }
}

pub struct FieldDef {
    pub(crate) name: String,
    pub(crate) type_: Type,
}

impl FieldDef {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    fn parse<'i>(&self, bytes: &'i [u8]) -> Result<(&'i [u8], Value), Error<'i>> {
        use nom::number::complete;

        match &self.type_ {
            Type::U8 => complete::le_u8(bytes).map(|(b, n)| (b, Value::U8(n))),
            Type::U32 => complete::le_u32(bytes).map(|(b, n)| (b, Value::U32(n))),
            Type::U64 => complete::le_u64(bytes).map(|(b, n)| (b, Value::U64(n))),
        }
        .map_err(Error::ValueParsing)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    U8,
    U32,
    U64,
}

impl Type {
    fn id(&self) -> &[u8] {
        match self {
            Type::U8 => &[0],
            Type::U32 => &[1],
            Type::U64 => &[2],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_fields() {
        let struct_def = StructDef {
            type_name: "Foo".to_string(),
            fields: vec![],
        };

        assert!(struct_def.parse(&[]).is_ok());
    }

    #[test]
    fn single_field() {
        let struct_def = StructDef {
            type_name: "Foo".to_string(),
            fields: vec![FieldDef {
                name: "bar".to_string(),
                type_: Type::U64,
            }],
        };

        let parsed = struct_def.parse(&[42, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(parsed["bar"], Value::U64(42));
    }

    #[test]
    fn multiple_fields() {
        let struct_def = StructDef {
            type_name: "Foo".to_string(),
            fields: vec![
                FieldDef {
                    name: "bar".to_string(),
                    type_: Type::U8,
                },
                FieldDef {
                    name: "baz".to_string(),
                    type_: Type::U8,
                },
                FieldDef {
                    name: "qux".to_string(),
                    type_: Type::U8,
                },
            ],
        };

        let parsed = struct_def.parse(&[42, 43, 44]).unwrap();
        assert_eq!(parsed["bar"], Value::U8(42));
        assert_eq!(parsed["baz"], Value::U8(43));
        assert_eq!(parsed["qux"], Value::U8(44));
    }

    #[test]
    fn too_few_bytes() {
        let struct_def = StructDef {
            type_name: "Foo".to_string(),
            fields: vec![FieldDef {
                name: "bar".to_string(),
                type_: Type::U32,
            }],
        };

        let result = struct_def.parse(&[42, 0]);
        assert!(result.is_err());
    }

    #[test]
    fn too_many_bytes() {
        let struct_def = StructDef {
            type_name: "Foo".to_string(),
            fields: vec![FieldDef {
                name: "bar".to_string(),
                type_: Type::U8,
            }],
        };

        let result = struct_def.parse(&[42, 0]);
        assert!(result.is_err());
    }

    #[cfg(test)]
    mod id {
        use super::*;

        #[test]
        fn is_different_with_different_field_names() {
            let struct_a = StructDef {
                type_name: "Foo".to_string(),
                fields: vec![FieldDef {
                    name: "bar".to_string(),
                    type_: Type::U8,
                }],
            };
            let struct_b = StructDef {
                type_name: "Foo".to_string(),
                fields: vec![FieldDef {
                    name: "baz".to_string(),
                    type_: Type::U8,
                }],
            };

            assert_ne!(struct_a.id(), struct_b.id());
        }

        #[test]
        fn is_different_with_different_field_types() {
            let struct_a = StructDef {
                type_name: "Foo".to_string(),
                fields: vec![FieldDef {
                    name: "bar".to_string(),
                    type_: Type::U8,
                }],
            };
            let struct_b = StructDef {
                type_name: "Foo".to_string(),
                fields: vec![FieldDef {
                    name: "bar".to_string(),
                    type_: Type::U32,
                }],
            };

            assert_ne!(struct_a.id(), struct_b.id());
        }

        #[test]
        fn is_same_with_same_fields() {
            let struct_a = StructDef {
                type_name: "Foo".to_string(),
                fields: vec![FieldDef {
                    name: "bar".to_string(),
                    type_: Type::U8,
                }],
            };
            let struct_b = StructDef {
                type_name: "Foo".to_string(),
                fields: vec![FieldDef {
                    name: "bar".to_string(),
                    type_: Type::U8,
                }],
            };

            assert_eq!(struct_a.id(), struct_b.id());
        }
    }
}
