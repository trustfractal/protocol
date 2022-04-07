use blake2::{Blake2b512, Digest};
use core::convert::TryInto;
use std::sync::Arc;

use crate::{Builder, Error, Object, Value};

pub type Id = [u8; 8];

#[derive(Debug, PartialEq, Eq)]
pub struct StructDef {
    pub(crate) type_name: String,
    pub(crate) fields: Vec<FieldDef>,
}

impl StructDef {
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn id(&self) -> [u8; 8] {
        let mut hasher = Blake2b512::new();

        hasher.update(&self.type_name);
        for field in &self.fields {
            hasher.update(&field.name);
            hasher.update(field.type_.id());
        }

        let digest = hasher.finalize();

        let bytes: &[u8] = &digest[0..8];
        // TODO(shelbyd): Make ID type and use #split method of GenericArray.
        bytes.try_into().expect("hash should always be > 8 bytes")
    }

    pub fn fields(&self) -> &[FieldDef] {
        self.fields.as_ref()
    }

    pub fn parse<'i>(&self, mut bytes: &'i [u8]) -> Result<(&'i [u8], Object), Error<'i>> {
        let mut values = Vec::with_capacity(self.fields.len());

        for field in &self.fields {
            let (new_bytes, value) = field.parse(bytes)?;
            bytes = new_bytes;
            values.push(value);
        }

        Ok((bytes, Object::new(self, values)))
    }

    pub fn builder(&self) -> Builder {
        Builder::new(self)
    }
}

#[derive(Debug, PartialEq, Eq)]
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
        self.type_.parse(bytes)
    }
}

use nom::{
    bytes::complete::{take, take_while},
    combinator::recognize,
    sequence::pair,
    IResult,
};

fn length_prefixed(b: &[u8]) -> IResult<&[u8], &[u8]> {
    let (b, len) = var_int(b)?;
    take(len)(b)
}

fn var_int(b: &[u8]) -> IResult<&[u8], usize> {
    let (new_b, int_bytes) = recognize(pair(take_while(|b| b & 128 > 0), take(1usize)))(b)?;

    let mut result: usize = 0;
    for (i, byte) in int_bytes.iter().enumerate() {
        let shift_by = 7 * i;
        let effective_byte = (byte & 0b0111_1111) as usize;

        let shifted = effective_byte << shift_by;
        if shifted >> shift_by != effective_byte {
            return Err(nom::Err::Error(nom::error::Error::new(
                b,
                nom::error::ErrorKind::TooLarge,
            )));
        }

        result |= shifted;
    }

    Ok((new_b, result))
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type<StructType = Arc<StructDef>> {
    Unit,
    Bool,
    U8,
    U32,
    U64,
    String,
    List(Box<Type<StructType>>),
    Struct(StructType),
}

impl Type {
    // Needs to be stable across versions of the code.
    fn id(&self) -> Vec<u8> {
        match self {
            Type::Unit => vec![5],
            Type::Bool => vec![7],
            Type::U8 => vec![0],
            Type::U32 => vec![1],
            Type::U64 => vec![2],
            Type::String => vec![3],
            Type::List(t) => {
                let mut res = vec![4];
                res.extend(t.id());
                res
            }
            Type::Struct(def) => {
                let mut res = vec![6];
                res.extend(def.id());
                res
            }
        }
    }

    fn parse<'i>(&self, bytes: &'i [u8]) -> Result<(&'i [u8], Value), Error<'i>> {
        use nom::number::complete;

        match self {
            Type::Unit => Ok((bytes, Value::Unit)),
            Type::Bool => {
                let (b, n) = complete::le_u8(bytes).map_err(Error::ValueParsing)?;
                let value = match n {
                    0 => false,
                    1 => true,
                    _ => {
                        return Err(Error::ValueParsing(nom::Err::Error(
                            nom::error::make_error(bytes, nom::error::ErrorKind::IsNot),
                        )));
                    }
                };
                Ok((b, Value::Bool(value)))
            }
            Type::U8 => complete::le_u8(bytes).map(|(b, n)| (b, Value::U8(n))),
            Type::U32 => complete::le_u32(bytes).map(|(b, n)| (b, Value::U32(n))),
            Type::U64 => complete::le_u64(bytes).map(|(b, n)| (b, Value::U64(n))),
            Type::String => {
                let (bytes, str_bytes) = length_prefixed(bytes).map_err(Error::ValueParsing)?;
                let s = std::str::from_utf8(str_bytes)?;

                Ok((bytes, Value::String(String::from(s))))
            }
            Type::List(t) => {
                let (bytes, mut list_bytes) =
                    length_prefixed(bytes).map_err(Error::ValueParsing)?;

                let mut items = Vec::new();
                while !list_bytes.is_empty() {
                    let (b, item) = t.parse(list_bytes)?;
                    list_bytes = b;
                    items.push(item);
                }
                Ok((bytes, Value::List(items)))
            }
            Type::Struct(def) => {
                let (bytes, obj) = def.parse(bytes)?;
                Ok((bytes, Value::Struct(obj)))
            }
        }
        .map_err(Error::ValueParsing)
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

        let parsed = struct_def.parse(&[42, 0, 0, 0, 0, 0, 0, 0]).unwrap().1;
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

        let parsed = struct_def.parse(&[42, 43, 44]).unwrap().1;
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
        assert!(matches!(result, Err(Error::ValueParsing(_))));
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

        #[test]
        fn is_different_with_different_list_types() {
            let struct_a = StructDef {
                type_name: "Foo".to_string(),
                fields: vec![FieldDef {
                    name: "bar".to_string(),
                    type_: Type::List(Box::new(Type::U8)),
                }],
            };
            let struct_b = StructDef {
                type_name: "Foo".to_string(),
                fields: vec![FieldDef {
                    name: "bar".to_string(),
                    type_: Type::List(Box::new(Type::U32)),
                }],
            };

            assert_ne!(struct_a.id(), struct_b.id());
        }

        #[test]
        fn is_different_with_different_struct_types() {
            let struct_a = StructDef {
                type_name: "Foo".to_string(),
                fields: vec![FieldDef {
                    name: "bar".to_string(),
                    type_: Type::Struct(Arc::new(StructDef {
                        type_name: "Bar".to_string(),
                        fields: vec![],
                    })),
                }],
            };
            let struct_b = StructDef {
                type_name: "Foo".to_string(),
                fields: vec![FieldDef {
                    name: "bar".to_string(),
                    type_: Type::Struct(Arc::new(StructDef {
                        type_name: "Baz".to_string(),
                        fields: vec![],
                    })),
                }],
            };

            assert_ne!(struct_a.id(), struct_b.id());
        }
    }

    #[cfg(test)]
    mod parsing {
        use super::*;

        #[cfg(test)]
        mod string {
            use super::*;

            #[test]
            fn empty_string() {
                let field = FieldDef {
                    name: "foo".to_string(),
                    type_: Type::String,
                };

                assert_eq!(
                    field.parse(&[0]).unwrap(),
                    (&[][..], Value::String("".to_string()))
                );
            }

            #[test]
            fn simple_string() {
                let field = FieldDef {
                    name: "foo".to_string(),
                    type_: Type::String,
                };

                assert_eq!(
                    field.parse(&[3, 65, 66, 67]).unwrap(),
                    (&[][..], Value::String("ABC".to_string()))
                );
            }

            #[test]
            fn non_utf8() {
                let field = FieldDef {
                    name: "foo".to_string(),
                    type_: Type::String,
                };

                assert!(matches!(
                    field.parse(&[4, 0, 159, 146, 150]),
                    Err(Error::InvalidUtf8(_))
                ));
            }

            #[test]
            fn keeps_extra_bytes() {
                let field = FieldDef {
                    name: "foo".to_string(),
                    type_: Type::String,
                };

                assert_eq!(
                    field.parse(&[2, 65, 66, 67, 68]).unwrap(),
                    (&[67, 68][..], Value::String("AB".to_string()))
                );
            }

            #[test]
            fn no_size_byte() {
                let field = FieldDef {
                    name: "foo".to_string(),
                    type_: Type::String,
                };

                assert!(matches!(field.parse(&[]), Err(Error::ValueParsing(_))));
            }

            #[test]
            fn fewer_bytes_than_size_says() {
                let field = FieldDef {
                    name: "foo".to_string(),
                    type_: Type::String,
                };

                assert!(matches!(
                    field.parse(&[3, 65, 66]),
                    Err(Error::ValueParsing(_))
                ));
            }

            #[test]
            fn struct_field() {
                let struct_ = Arc::new(StructDef {
                    type_name: "Foo".to_string(),
                    fields: vec![FieldDef {
                        name: "bar".to_string(),
                        type_: Type::U8,
                    }],
                });

                let field = FieldDef {
                    name: "foo".to_string(),
                    type_: Type::Struct(struct_),
                };

                let value = field.parse(&[42]).unwrap().1;
                let obj = value.as_object().unwrap();
                assert_eq!(obj["bar"].as_u8(), Some(42));
            }
        }

        #[cfg(test)]
        mod var_int {
            use super::*;

            #[test]
            fn zero() {
                assert_eq!(var_int(&[0]).unwrap(), (&[][..], 0));
            }

            #[test]
            fn one() {
                assert_eq!(var_int(&[1]).unwrap(), (&[][..], 1));
            }

            #[test]
            fn two_hundred_fifty_six() {
                let leading_bit = 0b1000_0000;
                assert_eq!(var_int(&[0 | leading_bit, 0b10]).unwrap(), (&[][..], 256));
            }

            #[test]
            fn trailing_bytes() {
                let leading_bit = 0b1000_0000;
                assert_eq!(var_int(&[0 | leading_bit, 0b10, 42]).unwrap().0, &[42][..]);
            }

            #[test]
            fn no_bytes() {
                assert!(matches!(var_int(&[]), Err(_)));
            }

            #[test]
            fn too_many_bits_for_usize() {
                let mut bytes = [128; 10];
                bytes[9] = 0b10;
                assert!(matches!(
                    var_int(&bytes),
                    Err(nom::Err::Error(e)) if e.code == nom::error::ErrorKind::TooLarge
                ));
            }

            #[test]
            fn just_inside_usize() {
                let mut bytes = [128; 10];
                bytes[9] = 0b1;
                assert!(var_int(&bytes).is_ok());
            }
        }
    }
}
