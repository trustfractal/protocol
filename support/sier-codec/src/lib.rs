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

    pub fn transform_serde_value<'s>(
        json_obj: &serde_json::Map<String, SerdeValue>,
    ) -> (String, Vec<FieldDef>, Vec<Value<'s>>) {
        let mut values = Vec::with_capacity(json_obj.len());
        let mut fields = Vec::with_capacity(json_obj.len());
        //TODO(melatron): Generate a name for the Json Struct Def so that there is no collision.
        let mut type_name = String::from("");
        for (key, value) in json_obj {
            let (field_type, value) = match value {
                SerdeValue::Null => (Type::Unit, Value::Unit),
                SerdeValue::Bool(_bool) => (Type::Unit, Value::Unit),
                SerdeValue::Number(number) => {
                    if let Some(n) = number.as_u64() {
                        (Type::U64, Value::U64(n))
                    } else {
                        (Type::Unit, Value::Unit)
                    }
                }
                SerdeValue::String(s) => (Type::String, Value::String(s.to_string())),
                SerdeValue::Array(_vec) => unimplemented!(), // (Type::Unit, Value::List(vec![])),
                SerdeValue::Object(_inner_json_obj) => unimplemented!(), //(Type::Unit, Value::Unit),
            };
            type_name.push_str(key.as_str());
            values.push(value);
            fields.push(FieldDef {
                name: key.to_string(),
                type_: field_type,
            })
        }
        (type_name, fields, values)
    }

    pub fn parse_json<'i>(&mut self, file_json_contents: &'i str) -> Result<Object, Error<'i>> {
        let json: SerdeValue =
            serde_json::from_str(file_json_contents).or(Err(Error::InvalidJson))?;

        let (type_name, fields, values) =
            Parser::transform_serde_value(json.as_object().ok_or(Error::InvalidJson)?);
        let json_def = StructDef { type_name, fields };
        let rw_def = Arc::new(json_def);
        let id = rw_def.id();
        let existing = self.structs.insert(id, rw_def);
        if let Some(s) = existing {
            return Err(Error::DuplicateStructDef(s.type_name().to_string()));
        }

        Ok(Object::new(self.structs.get(&id).unwrap().as_ref(), values))
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
