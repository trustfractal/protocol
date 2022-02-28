use std::{collections::HashMap, sync::Arc};

mod builder;
use builder::Builder;

mod definition_parser;

mod object;
use object::{Object, Value};

mod schema;
use schema::{FieldDef, Id, StructDef, Type};

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
