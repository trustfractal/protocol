use core::ops::Index;

use crate::schema::{StructDef, Type};

#[derive(Debug, PartialEq)]
pub struct Object<'s> {
    pub(crate) schema: &'s StructDef,
    pub(crate) values: Vec<Value<'s>>,
}

impl Object<'_> {
    pub fn schema(&self) -> &StructDef {
        self.schema
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend(self.schema.id().iter().cloned());
        for value in &self.values {
            result.extend(value.serialize());
        }
        result
    }
}

impl<'s> Index<&'_ str> for Object<'s> {
    type Output = Value<'s>;

    fn index(&self, field_name: &str) -> &Self::Output {
        let index = self
            .schema
            .fields()
            .iter()
            .position(|f| f.name() == field_name)
            .unwrap_or_else(|| panic!("no field with name '{}'", field_name));
        &self.values[index]
    }
}

#[derive(Debug, PartialEq)]
pub enum Value<'s> {
    Unit,
    U8(u8),
    U32(u32),
    U64(u64),
    String(String),
    List(Vec<Value<'s>>),
    Struct(Object<'s>)
}

impl<'s> From<()> for Value<'s> {
    fn from(_: ()) -> Value<'s> {
        Value::Unit
    }
}

impl<'s> From<u8> for Value<'s> {
    fn from(v: u8) -> Value<'s> {
        Value::U8(v)
    }
}

impl<'s> From<u32> for Value<'s> {
    fn from(v: u32) -> Value<'s> {
        Value::U32(v)
    }
}

impl<'s> From<u64> for Value<'s> {
    fn from(v: u64) -> Value<'s> {
        Value::U64(v)
    }
}

impl<'s> From<String> for Value<'s> {
    fn from(v: String) -> Value<'s> {
        Value::String(v)
    }
}

impl<'s, T> From<Vec<T>> for Value<'s>
where
    T: Into<Value<'s>>,
{
    fn from(items: Vec<T>) -> Value<'s> {
        Value::List(items.into_iter().map(Into::into).collect())
    }
}

impl<'s> From<Object<'s>> for Value<'s> {
    fn from(v: Object) -> Value {
        Value::Struct(v)
    }
}

impl<'s> Value<'s> {
    pub fn as_unit(&self) -> Option<()> {
        match self {
            Value::Unit => Some(()),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> Option<u8> {
        match self {
            Value::U8(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Value::U32(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::U64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(items) => Some(items),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&Object> {
        match self {
            Value::Struct(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Value::Unit => Vec::new(),
            Value::U8(v) => Vec::from(v.to_le_bytes()),
            Value::U32(v) => Vec::from(v.to_le_bytes()),
            Value::U64(v) => Vec::from(v.to_le_bytes()),
            Value::String(v) => var_int(v.len()).into_iter().chain(v.bytes()).collect(),
            Value::List(items) => {
                let item_bytes = items.iter().flat_map(|i| i.serialize()).collect::<Vec<_>>();
                var_int(item_bytes.len())
                    .into_iter()
                    .chain(item_bytes)
                    .collect()
            },
            Value::Struct(obj) => obj.serialize()
        }
    }

    pub fn assignable(&self, type_: &Type) -> Result<(), (Type, Type)> {
        match (self, type_) {
            (Value::Unit, Type::Unit) => Ok(()),
            (Value::U8(_), Type::U8) => Ok(()),
            (Value::U32(_), Type::U32) => Ok(()),
            (Value::U64(_), Type::U64) => Ok(()),
            (Value::String(_), Type::String) => Ok(()),
            (Value::List(items), Type::List(inner)) => {
                items.iter().try_for_each(|i| i.assignable(inner))
            },
            (Value::Struct(_), Type::Struct) => Ok(()), // TODO (melatron): Chceck if the two structure definitions for Object and Type::Struct(&Object) are the same
            (v, t) => Err((t.clone(), v.type_())),
        }
    }

    fn type_(&self) -> Type {
        match self {
            Value::Unit => Type::Unit,
            Value::U8(_) => Type::U8,
            Value::U32(_) => Type::U32,
            Value::U64(_) => Type::U64,
            Value::String(_) => Type::String,
            Value::List(items) => {
                let item_type = items
                    .first()
                    .map(|i| i.type_())
                    .unwrap_or_else(|| Type::Unit);
                Type::List(Box::new(item_type))
            },
            Value::Struct(_) => Type::Struct // TODO (melatron): provide the obj to the Type::Struct
        }
    }
}

fn var_int(val: usize) -> Vec<u8> {
    // VarInts use 8 bits to encode 7 bits, so need to be multiplied by 8/7.
    let capacity = core::mem::size_of::<usize>() * 8 / 7 + 1;
    let mut result = Vec::with_capacity(capacity);

    let mut remaining = val;
    loop {
        let mut byte: u8 = (remaining as u8) & 0b01111111;

        remaining >>= 7;

        if remaining > 0 {
            byte |= 0b10000000;
            result.push(byte);
        } else {
            result.push(byte);
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod value_serialize {
        use super::*;

        #[test]
        fn u8_is_byte() {
            assert_eq!(Value::U8(42).serialize(), vec![42]);
        }

        #[test]
        fn u32_is_le_bytes() {
            assert_eq!(Value::U32(42).serialize(), vec![42, 0, 0, 0]);
        }

        #[test]
        fn u64_is_le_bytes() {
            assert_eq!(Value::U64(42).serialize(), vec![42, 0, 0, 0, 0, 0, 0, 0]);
        }

        #[test]
        fn string_is_length_prefixed() {
            assert_eq!(
                Value::String(String::from("foo")).serialize(),
                vec![3, 102, 111, 111]
            );
        }

        #[test]
        fn string_is_byte_length_prefixed() {
            let s = "⡌⠁⠧⠑";
            assert_eq!(
                Value::String(String::from(s)).serialize()[0],
                s.as_bytes().len() as u8
            );
        }

        #[test]
        fn list_is_length_prefixed() {
            assert_eq!(
                Value::List(vec![Value::U8(4), Value::U8(2)]).serialize(),
                vec![2, 4, 2]
            );
        }

        #[test]
        fn list_is_byte_length_prefixed() {
            assert_eq!(
                Value::List(vec![Value::U32(4), Value::U32(2)]).serialize(),
                vec![8, 4, 0, 0, 0, 2, 0, 0, 0]
            );
        }
    }

    #[cfg(test)]
    mod assignable {
        use super::*;

        #[test]
        fn list_with_different_types() {
            let list = Value::List(vec![Value::U8(4), Value::U32(32)]);
            assert_eq!(
                list.assignable(&Type::List(Box::new(Type::U8))),
                Err((Type::U8, Type::U32))
            );
        }

        #[test]
        fn empty_list_expected_against_primitive() {
            let list = Value::List(vec![]);
            assert_eq!(
                list.assignable(&Type::U8),
                Err((Type::U8, Type::List(Box::new(Type::Unit))))
            );
        }
    }

    #[cfg(test)]
    mod var_int {
        use super::*;

        #[test]
        fn zero_is_zero() {
            assert_eq!(var_int(0), vec![0]);
        }

        #[test]
        fn one_is_one() {
            assert_eq!(var_int(1), vec![1]);
        }

        #[test]
        fn two_bytes() {
            assert_eq!(var_int(128), vec![128, 1]);
        }
    }
}
