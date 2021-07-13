use core::ops::Index;

use crate::schema::{StructDef, Type};

pub struct Object<'s> {
    pub(crate) schema: &'s StructDef,
    pub(crate) values: Vec<Value>,
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
    type Output = Value;

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

macro_rules! impl_value_for_types {
    ($({$rust_type:ty, $variant:ident},)*) => {
        #[derive(Debug, PartialEq, Eq)]
        pub enum Value {
            $(
                $variant($rust_type),
            )*
        }

        $(
            impl From<$rust_type> for self::Value {
                fn from(v: $rust_type) -> Self {
                    Self::$variant(v)
                }
            }
        )*

        impl self::Value {
            pub fn type_(&self) -> Type {
                match self {
                    $(
                        self::Value::$variant(_) => crate::Type::$variant,
                    )*
                }
            }
        }
    }
}

impl_value_for_types!(
    {u8, U8},
    {u32, U32},
    {u64, U64},
    {String, String},
);

impl Value {
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

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Value::U8(v) => Vec::from(v.to_le_bytes()),
            Value::U32(v) => Vec::from(v.to_le_bytes()),
            Value::U64(v) => Vec::from(v.to_le_bytes()),
            Value::String(v) => var_int(v.len()).into_iter().chain(v.bytes()).collect(),
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
