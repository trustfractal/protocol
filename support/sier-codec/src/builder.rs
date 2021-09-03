use crate::{FieldDef, Object, StructDef, Type, Value};

use std::collections::HashMap;

pub struct Builder<'s> {
    struct_def: &'s StructDef,
    field_values: HashMap<String, Value>,
}

impl<'s> Builder<'s> {
    pub(crate) fn new(struct_def: &'s StructDef) -> Self {
        Builder {
            struct_def,
            field_values: HashMap::new(),
        }
    }

    pub fn set(mut self, field_name: &str, value: impl Into<Value>) -> Self {
        self.field_values
            .insert(field_name.to_string(), value.into());
        self
    }

    pub fn try_build(mut self) -> Result<Object<'s>, BuildError> {
        let values = self
            .struct_def
            .fields
            .iter()
            .map(|field| self.value_for_field(field))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Object {
            schema: self.struct_def,
            values,
        })
    }

    fn value_for_field(&mut self, field: &FieldDef) -> Result<Value, BuildError> {
        let value = self
            .field_values
            .remove(&field.name)
            .ok_or_else(|| BuildError::MissingField(self.field_name(field)))?;

        if value.type_() != field.type_ {
            return Err(BuildError::IncorrectType {
                field: self.field_name(field),
                expected: field.type_.clone(),
                got: value.type_(),
            });
        }

        Ok(value)
    }

    fn field_name(&self, field: &FieldDef) -> String {
        format!("{}.{}", self.struct_def.type_name, field.name)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BuildError {
    MissingField(String),
    IncorrectType {
        field: String,
        expected: Type,
        got: Type,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_struct() {
        let def = StructDef {
            type_name: "Foo".to_string(),
            fields: vec![],
        };

        let obj = def.builder().try_build();

        assert!(obj.is_ok());
    }

    #[test]
    fn missing_field() {
        let def = StructDef {
            type_name: "Foo".to_string(),
            fields: vec![FieldDef {
                name: "bar".to_string(),
                type_: Type::U8,
            }],
        };

        let obj = def.builder().try_build();

        assert_eq!(
            obj.err(),
            Some(BuildError::MissingField("Foo.bar".to_string()))
        );
    }

    #[test]
    fn provided_field() {
        let def = StructDef {
            type_name: "Foo".to_string(),
            fields: vec![FieldDef {
                name: "bar".to_string(),
                type_: Type::U8,
            }],
        };

        let obj = def.builder().set("bar", 42u8).try_build();

        assert!(obj.is_ok());
        let obj = obj.unwrap();

        assert_eq!(obj["bar"].as_u8(), Some(42));
    }

    #[test]
    fn bad_type_for_field() {
        let def = StructDef {
            type_name: "Foo".to_string(),
            fields: vec![FieldDef {
                name: "bar".to_string(),
                type_: Type::U8,
            }],
        };

        let obj = def.builder().set("bar", 42u32).try_build();

        assert_eq!(
            obj.err(),
            Some(BuildError::IncorrectType {
                field: "Foo.bar".to_string(),
                expected: Type::U8,
                got: Type::U32
            })
        );
    }

    #[test]
    fn second_set_overrides_first() {
        let def = StructDef {
            type_name: "Foo".to_string(),
            fields: vec![FieldDef {
                name: "bar".to_string(),
                type_: Type::U8,
            }],
        };

        let obj = def.builder().set("bar", 42u8).set("bar", 43u8).try_build();

        assert!(obj.is_ok());
        let obj = obj.unwrap();

        assert_eq!(obj["bar"].as_u8(), Some(43));
    }
}
