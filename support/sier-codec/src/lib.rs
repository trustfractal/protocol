use std::collections::HashMap;

mod definition_parser;
use definition_parser::parse;

mod object;
use object::{Object, Value};

mod schema;
use schema::{Id, StructDef};

pub struct Parser {
    structs: HashMap<Id, StructDef>,
}

impl Parser {
    pub fn add_file_defs<'i>(&mut self, file_contents: &'i str) -> Result<(), Error<'i>> {
        for schema in parse(file_contents)? {
            self.structs.insert(schema.id(), schema);
        }

        Ok(())
    }

    pub fn parse<'i>(&self, bytes: &'i [u8]) -> Result<Object, Error<'i>> {
        use core::convert::TryInto;

        let id = bytes[0..8].try_into().map_err(|_| Error::Incomplete)?;
        let schema = self.structs.get(&id).ok_or(Error::MissingId(id))?;

        let bytes = &bytes[8..];
        schema.parse(bytes)
    }

    pub fn struct_def(&self, name: &str) -> Option<&StructDef> {
        self.structs
            .values()
            .filter(|s| s.type_name() == name)
            .next()
    }
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            structs: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum Error<'i> {
    Incomplete,
    MissingId(Id),
    DefinitionParsing(nom::Err<nom::error::Error<&'i str>>),
    ValueParsing(nom::Err<nom::error::Error<&'i [u8]>>),
    UnresolvedType(String),
    DuplicateField(String),
    TooManyBytes,
}
