use std::{collections::HashMap, sync::Arc};

mod builder;
use builder::Builder;

mod definition_parser;

mod object;
use object::{Object, Value};

mod schema;
use schema::{FieldDef, Id, StructDef, Type};

pub mod json;

use serde_json::Value as SerdeValue;
use thiserror::Error as ThisError;

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

    pub fn json_str<'a>(
        &'a self,
        file_json_contents: &str,
        def: &'a Arc<StructDef>,
    ) -> Result<Object, Error<'a>> {
        let json: SerdeValue =
            serde_json::from_str(file_json_contents).map_err(|_| Error::InvalidJson)?;

        let obj = json::transform_serde_obj(json.as_object().ok_or(Error::InvalidJson)?, def)?;
        Ok(obj)
    }
}

#[derive(ThisError, Debug, PartialEq)]
pub enum Error<'i> {
    #[error("ID not found: {:?}", [..])]
    MissingId(Id),
    #[error("Could not parse definition: {0}")]
    DefinitionParsing(nom::Err<nom::error::Error<&'i str>>),
    #[error("Could not parse value: {0}")]
    ValueParsing(nom::Err<nom::error::Error<&'i [u8]>>),
    #[error("Could not find type: {0}")]
    UnresolvedType(String),
    #[error("Duplicate field: {0}")]
    DuplicateField(String),
    #[error("Struct already defined: {0}")]
    DuplicateStructDef(String),
    #[error("Unknown type: {0}")]
    UnrecognizedType(String),
    #[error("Too few bytes")]
    TooFewBytes,
    #[error("Too many bytes")]
    TooManyBytes,
    #[error("Invalid UTF8")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    #[error("Invalid JSON")]
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
