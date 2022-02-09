use crate::{Error, Object, StructDef, Type, Value};
use serde_json::Value as SerdeValue;
use std::sync::Arc;

pub fn transform_serde_value<'a>(
    value: &SerdeValue,
    type_: &'a Type,
) -> Result<Value<'a>, Error<'a>> {
    match value {
        SerdeValue::Null => unimplemented!(),
        SerdeValue::Bool(_bool) => unimplemented!(),
        SerdeValue::Number(number) => {
            if type_ == &Type::U64 {
                if let Some(n) = number.as_u64() {
                    Ok(Value::U64(n))
                } else {
                    unimplemented!()
                }
            } else {
                Err(Error::MismatchTypeFromJson(type_))
            }
        }
        SerdeValue::String(s) => {
            if type_ == &Type::String {
                Ok(Value::String(s.to_string()))
            } else {
                Err(Error::MismatchTypeFromJson(type_))
            }
        }
        SerdeValue::Array(vec) => {
            if let Type::List(arr_type) = type_ {
                let mut list = vec![];
                for arr_value in vec {
                    let val = transform_serde_value(arr_value, arr_type)?;
                    list.push(val);
                }
                Ok(Value::List(list))
            } else {
                Err(Error::MismatchTypeFromJson(type_))
            }
        }
        SerdeValue::Object(inner_json_obj) => {
            if let Type::Struct(struct_type) = type_ {
                Ok(Value::Struct(transform_serde_obj(
                    inner_json_obj,
                    struct_type,
                )?))
            } else {
                Err(Error::MismatchTypeFromJson(type_))
            }
        }
    }
}

pub fn transform_serde_obj<'a>(
    json_obj: &serde_json::Map<String, SerdeValue>,
    def: &'a Arc<StructDef>,
) -> Result<Object<'a>, Error<'a>> {
    let mut values = Vec::with_capacity(json_obj.len());
    for field_def in def.fields().iter() {
        let sier_value = transform_serde_value(
            json_obj.get(field_def.name()).unwrap_or(&SerdeValue::Null),
            field_def.type_(),
        )?;
        values.push(sier_value);
    }

    Ok(Object::new(def.as_ref(), values))
}
