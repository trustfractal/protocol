use std::{collections::HashMap, sync::Arc};

mod builder;
use builder::Builder;

mod definition_parser;

mod object;
use object::{Object, Value};

mod schema;
use schema::{FieldDef, Id, StructDef, Type};

use serde_json::Value as SerdeValue;

#[derive(Debug, Default)]
pub struct Parser {
    structs: HashMap<Id, Arc<StructDef>>,
}

impl Parser {
    pub fn add_file_defs<'i>(&mut self, file_contents: &'i str) -> Result<(), Error<'i>> {
        let mut remaining_contents = file_contents;

        while let (c, Some(def)) = definition_parser::next_def(remaining_contents, self)? {
            let existing = self.structs.insert(def.id(), Arc::new(def));
            if let Some(s) = existing {
                return Err(Error::DuplicateStructDef(s.type_name().to_string()));
            }
            remaining_contents = c;
        }

        Ok(())
    }

    pub fn parse<'i>(&self, bytes: &'i [u8]) -> Result<Object, Error<'i>> {
        use core::convert::TryInto;

        let id = bytes[0..8].try_into().map_err(|_| Error::TooFewBytes)?;
        let schema = self.structs.get(&id).ok_or(Error::MissingId(id))?;

        let bytes = &bytes[8..];
        let (bytes, obj) = schema.parse(bytes)?;
        if !bytes.is_empty() {
            return Err(Error::TooManyBytes);
        }
        Ok(obj)
    }

    pub fn struct_def(&self, name: &str) -> Option<&Arc<StructDef>> {
        self.structs.values().find(|s| s.type_name() == name)
    }

    fn transform_serde_value<'a>(
        &'a self,
        value: &SerdeValue,
        type_: &'a Type,
    ) -> Result<Value<'a>, Error<'a>> {
        match value {
            SerdeValue::Null => Ok(Value::Unit),
            //TODO(melatron): Implement a boolean type for sier
            SerdeValue::Bool(_bool) => Ok(Value::Unit),
            SerdeValue::Number(number) => {
                if let Some(n) = number.as_u64() {
                    Ok(Value::U64(n))
                } else {
                    Err(Error::InvalidJson)
                }
            }
            SerdeValue::String(s) => Ok(Value::String(s.to_string())),
            SerdeValue::Array(vec) => {
                if let Type::List(arr_type) = type_ {
                    let mut list = vec![];
                    for arr_value in vec {
                        let val = self.transform_serde_value(arr_value, arr_type)?;
                        list.push(val);
                    }
                    Ok(Value::List(list))
                } else {
                    Err(Error::InvalidJson)
                }
            }
            SerdeValue::Object(inner_json_obj) => {
                if let Type::Struct(struct_type) = type_ {
                    Ok(Value::Struct(
                        self.transform_serde_obj(inner_json_obj, struct_type)?,
                    ))
                } else {
                    Err(Error::InvalidJson)
                }
            }
        }
    }

    fn transform_serde_obj<'a>(
        &'a self,
        json_obj: &serde_json::Map<String, SerdeValue>,
        def: &'a Arc<StructDef>,
    ) -> Result<Object, Error<'a>> {
        let mut values = Vec::with_capacity(json_obj.len());
        for field_def in def.fields().iter() {
            let sier_value = self.transform_serde_value(
                json_obj.get(field_def.name()).ok_or(Error::InvalidJson)?,
                field_def.type_(),
            )?;
            values.push(sier_value);
        }

        Ok(Object::new(def.as_ref(), values))
    }

    pub fn parse_json<'a>(
        &'a self,
        file_json_contents: &str,
        def: &'a Arc<StructDef>,
    ) -> Result<Object, Error<'a>> {
        let json: SerdeValue =
            serde_json::from_str(file_json_contents).or(Err(Error::InvalidJson))?;

        let obj = self.transform_serde_obj(json.as_object().ok_or(Error::InvalidJson)?, def)?;
        Ok(obj)
    }
}

#[derive(Debug, PartialEq)]
pub enum Error<'i> {
    MissingId(Id),
    DefinitionParsing(nom::Err<nom::error::Error<&'i str>>),
    ValueParsing(nom::Err<nom::error::Error<&'i [u8]>>),
    UnresolvedType(String),
    DuplicateField(String),
    DuplicateStructDef(String),
    UnrecognizedType(String),
    TooFewBytes,
    TooManyBytes,
    InvalidUtf8(std::str::Utf8Error),
    InvalidJson,
}

#[cfg(test)]
mod tests {
    use super::*;
    const DUPLICATE_STRUCT: &'static str = r#"
    struct Foo {
        foo :u8;
    }

    struct Foo {
        foo :u8;
    }
    "#;
    #[test]
    fn duplicate_struct() {
        let mut parser = Parser::default();
        let result = parser.add_file_defs(DUPLICATE_STRUCT);
        assert_eq!(
            result.unwrap_err(),
            Error::DuplicateStructDef("Foo".to_string())
        );
    }

    const UNDECLARED_STRUCT: &'static str = r#"
    struct Foo {
        foo :Bar;
    }
    "#;
    #[test]
    fn undeclared_struct_field() {
        let mut parser = Parser::default();
        let result = parser.add_file_defs(UNDECLARED_STRUCT);
        assert_eq!(
            result.unwrap_err(),
            Error::UnrecognizedType("Bar".to_string())
        );
    }
}
