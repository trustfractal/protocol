use std::collections::{HashMap, HashSet};

mod builder;
use builder::Builder;

mod definition_parser;
use definition_parser::parse;

mod object;
use object::{Object, Value};

mod schema;
use schema::{FieldDef, Id, StructDef, Type};

#[derive(Debug)]
pub struct Parser {
    struct_names: HashSet<String>,
    structs: HashMap<Id, StructDef>,
}

impl Parser {
    pub fn add_file_defs<'i>(&mut self, file_contents: &'i str) -> Result<(), Error<'i>> {
        for schema in parse(file_contents, &mut self.struct_names)? {
            self.structs.insert(schema.id(), schema);
        }

        Ok(())
    }

    pub fn parse<'i>(&self, bytes: &'i [u8]) -> Result<Object, Error<'i>> {
        use core::convert::TryInto;

        let id = bytes[0..8].try_into().map_err(|_| Error::TooFewBytes)?;
        let schema = self.structs.get(&id).ok_or(Error::MissingId(id))?;

        let bytes = &bytes[8..];
        schema.parse(bytes)
    }

    pub fn struct_def(&self, name: &str) -> Option<&StructDef> {
        self.structs.values().find(|s| s.type_name() == name)
    }
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            struct_names: HashSet::new(),
            structs: HashMap::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error<'i> {
    MissingId(Id),
    DefinitionParsing(nom::Err<nom::error::Error<&'i str>>),
    ValueParsing(nom::Err<nom::error::Error<&'i [u8]>>),
    UnresolvedType(String),
    DuplicateField(String),
    DuplicateStruct(String),
    UndeclaredStructField(String),
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
            Error::DuplicateStruct("Foo".to_string())
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
            Error::UndeclaredStructField("Bar".to_string())
        );
    }
}
