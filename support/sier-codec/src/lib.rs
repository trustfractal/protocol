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
        &mut self,
        value: &SerdeValue,
    ) -> Result<(Type, Value<'a>), Error<'a>> {
        match value {
            SerdeValue::Null => Ok((Type::Unit, Value::Unit)),
            SerdeValue::Bool(_bool) => Ok((Type::Unit, Value::Unit)),
            SerdeValue::Number(number) => {
                if let Some(n) = number.as_u64() {
                    Ok((Type::U64, Value::U64(n)))
                } else {
                    Ok((Type::Unit, Value::Unit))
                }
            }
            SerdeValue::String(s) => Ok((Type::String, Value::String(s.to_string()))),
            SerdeValue::Array(vec) => {
                let mut list = vec![];
                let mut arr_type = Type::Unit;
                for arr_value in vec {
                    let (val_type, val) = self.transform_serde_value(arr_value)?;
                    arr_type = val_type;
                    list.push(val);
                }
                Ok((Type::List(Box::new(arr_type)), Value::List(list)))
            }
            SerdeValue::Object(inner_json_obj) => {
                let (_object, def) = self.transform_serde_obj(inner_json_obj)?;
                Ok((
                    Type::Struct(def),
                    Value::Unit,
                    //TODO(melatron): Figure how to handle the lifetime of object - Value::Struct(object),
                ))
            }
        }
    }

    fn transform_serde_obj<'a>(
        &mut self,
        json_obj: &serde_json::Map<String, SerdeValue>,
    ) -> Result<(Object, Arc<StructDef>), Error<'a>> {
        let mut values = Vec::with_capacity(json_obj.len());
        let mut fields = Vec::with_capacity(json_obj.len());
        let mut type_name = String::from("");
        for (key, value) in json_obj {
            let (field_type, sier_value) = self.transform_serde_value(value)?;
            type_name.push_str(key.as_str());
            values.push(sier_value);
            fields.push(FieldDef {
                name: key.to_string(),
                type_: field_type,
            })
        }

        let json_def = StructDef { type_name, fields };
        let id = json_def.id();
        let existing = self.structs.insert(id, Arc::new(json_def));
        if let Some(s) = existing {
            return Err(Error::DuplicateStructDef(s.type_name().to_string()));
        }
        let def = self
            .structs
            .get(&id)
            .expect("Check for existance is already done.");
        Ok((Object::new(def.as_ref(), values), def.clone()))
    }

    pub fn parse_json<'a>(&mut self, file_json_contents: &'a str) -> Result<Object, Error<'a>> {
        let json: SerdeValue =
            serde_json::from_str(file_json_contents).or(Err(Error::InvalidJson))?;

        let (obj, _) = self.transform_serde_obj(json.as_object().ok_or(Error::InvalidJson)?)?;
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
