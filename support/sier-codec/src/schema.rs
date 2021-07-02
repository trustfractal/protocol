use blake2::{Blake2b, Digest};
use core::convert::TryInto;

pub type Id = [u8; 8];

pub struct StructDef {
    type_name: String,
}

impl StructDef {
    pub fn type_name(&self) -> &str {
        self.type_name.as_ref()
    }

    pub fn id(&self) -> [u8; 8] {
        let digest = Blake2b::digest(self.type_name.as_bytes());
        let bytes: &[u8] = &digest[0..8];
        bytes.try_into().expect("hash should always be > 8 bytes")
    }
}

use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, multispace0, multispace1},
    multi::separated_list0,
    IResult,
};

pub fn parse(s: &str) -> Result<Vec<StructDef>, nom::Err<nom::error::Error<&str>>> {
    let (s, _) = multispace1(s)?;
    let (_, structs) = separated_list0(multispace0, struct_def)(s)?;

    Ok(structs)
}

fn struct_def(s: &str) -> IResult<&str, StructDef> {
    let (s, _) = tag("struct")(s)?;
    let (s, _) = multispace1(s)?;
    let (s, ident) = ident(s)?;
    let (s, _) = multispace1(s)?;
    let (s, _) = tag("{}")(s)?;

    Ok((
        s,
        StructDef {
            type_name: ident.to_string(),
        },
    ))
}

fn ident(s: &str) -> IResult<&str, &str> {
    alphanumeric1(s)
}
