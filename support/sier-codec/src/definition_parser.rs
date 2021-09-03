use crate::{
    schema::{FieldDef, StructDef, Type},
    Error,
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, multispace0, multispace1},
    multi::{many0, separated_list0},
    IResult,
};

#[derive(Debug)]
struct ParsedStruct<'i> {
    type_name: &'i str,
    fields: Vec<ParsedField<'i>>,
}

#[derive(Debug)]
struct ParsedField<'i> {
    name: &'i str,
    type_: TypeDef<'i>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TypeDef<'i> {
    Primitive(Type),
    Generic(&'i str, Box<TypeDef<'i>>),
    Unresolved(&'i str),
}

impl<'i> ParsedStruct<'i> {
    fn compile(self) -> Result<StructDef, Error<'i>> {
        use std::collections::HashSet;

        let mut seen = HashSet::with_capacity(self.fields.len());
        for field in &self.fields {
            if seen.contains(field.name) {
                return Err(Error::DuplicateField(field.name.to_string()));
            }
            seen.insert(field.name);
        }

        Ok(StructDef {
            type_name: self.type_name.to_string(),
            fields: self
                .fields
                .into_iter()
                .map(|f| {
                    Ok(FieldDef {
                        name: f.name.to_string(),
                        type_: f.type_.resolve()?,
                    })
                })
                .collect::<Result<_, _>>()?,
        })
    }
}

impl<'i> TypeDef<'i> {
    fn resolve(self) -> Result<Type, Error<'i>> {
        match self {
            TypeDef::Primitive(t) => Ok(t),
            TypeDef::Generic("List", t) => Ok(Type::List(Box::new(t.resolve()?))),
            TypeDef::Unresolved(name) | TypeDef::Generic(name, _) => {
                Err(Error::UnresolvedType(name.to_string()))
            }
        }
    }
}

pub fn parse(s: &str) -> Result<Vec<StructDef>, Error> {
    let (s, _) = multispace0(s).map_err(Error::DefinitionParsing)?;
    let (_, structs) =
        separated_list0(multispace0, struct_def)(s).map_err(Error::DefinitionParsing)?;

    structs
        .into_iter()
        .map(ParsedStruct::compile)
        .collect::<Result<_, _>>()
}

fn struct_def(s: &str) -> IResult<&str, ParsedStruct> {
    let (s, _) = tag("struct")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, ident) = ident(s)?;
    let (s, _) = multispace1(s)?;
    let (s, _) = tag("{")(s)?;
    let (s, fields) = many0(field)(s)?;
    let (s, _) = tag("}")(s)?;

    Ok((
        s,
        ParsedStruct {
            type_name: ident,
            fields,
        },
    ))
}

fn ident(s: &str) -> IResult<&str, &str> {
    alphanumeric1(s)
}

fn field(s: &str) -> IResult<&str, ParsedField> {
    let (s, _) = multispace0(s)?;
    let (s, name) = ident(s)?;
    let (s, _) = multispace1(s)?;
    let (s, _) = tag(":")(s)?;
    let (s, type_) = type_(s)?;
    let (s, _) = tag(";")(s)?;
    let (s, _) = multispace0(s)?;
    Ok((s, ParsedField { name, type_ }))
}

fn type_(s: &str) -> IResult<&str, TypeDef> {
    alt((generic_type, leaf_type))(s)
}

fn generic_type(s: &str) -> IResult<&str, TypeDef> {
    let (s, outer_type) = ident(s)?;
    let (s, _) = tag("<")(s)?;
    let (s, inner_type) = type_(s)?;
    let (s, _) = tag(">")(s)?;
    Ok((s, TypeDef::Generic(outer_type, Box::new(inner_type))))
}

fn leaf_type(s: &str) -> IResult<&str, TypeDef> {
    let (s, type_str) = ident(s)?;
    let as_type = match type_str {
        "u8" => TypeDef::Primitive(Type::U8),
        "u32" => TypeDef::Primitive(Type::U32),
        "u64" => TypeDef::Primitive(Type::U64),
        "string" => TypeDef::Primitive(Type::String),
        v => TypeDef::Unresolved(v),
    };
    Ok((s, as_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_struct() {
        let (_, struct_) = struct_def("struct Foo {}").unwrap();

        assert_eq!(struct_.type_name, "Foo");
        assert_eq!(struct_.fields.len(), 0);
    }

    #[test]
    fn single_field() {
        let (_, struct_) = struct_def("struct Foo { bar :u64; }").unwrap();

        let fields = struct_.fields;
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "bar");
        assert_eq!(fields[0].type_, TypeDef::Primitive(Type::U64));
    }

    #[test]
    fn duplicate_fields() {
        let result = parse("struct Foo { bar :u64; bar :u64; }");
        assert!(result.is_err());
    }
}
