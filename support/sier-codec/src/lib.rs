use std::collections::HashMap;

mod schema;
use schema::{Id, StructDef};

pub struct Parser {
    structs: HashMap<Id, StructDef>,
}

impl Parser {
    pub fn add_file_defs<'i>(&mut self, file_contents: &'i str) -> Result<(), Error<'i>> {
        for schema in schema::parse(file_contents).map_err(Error::Parsing)? {
            self.structs.insert(schema.id(), schema);
        }

        Ok(())
    }

    pub fn parse(&self, bytes: &[u8]) -> Result<Object, Error> {
        use core::convert::TryInto;

        let id = bytes[0..8].try_into().map_err(|_| Error::Incomplete)?;
        let schema = self.structs.get(&id).ok_or(Error::MissingId(id))?;

        Ok(Object { schema })
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
    Parsing(nom::Err<nom::error::Error<&'i str>>),
}

pub struct Object<'s> {
    schema: &'s StructDef,
}

impl Object<'_> {
    pub fn schema(&self) -> &StructDef {
        self.schema
    }
}
