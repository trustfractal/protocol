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
}
