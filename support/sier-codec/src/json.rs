use crate::{Error, Object, StructDef, Type, Value};
use serde_json::Value as SerdeValue;
use std::sync::Arc;

pub fn parse_serde_value<'a>(value: &SerdeValue, type_: &'a Type) -> Result<Value<'a>, Error<'a>> {
    match value {
        SerdeValue::Null => unimplemented!(),
        SerdeValue::Bool(_bool) => unimplemented!(),
        SerdeValue::Number(number) => {
            if *type_ != Type::U64 {
                return Err(Error::InvalidJson);
            }

            let n = number.as_u64().unwrap_or_else(|| unimplemented!());
            Ok(Value::U64(n))
        }
        SerdeValue::String(s) => {
            if *type_ != Type::String {
                return Err(Error::InvalidJson);
            }

            Ok(Value::String(s.to_string()))
        }
        SerdeValue::Array(vec) => {
            if let Type::List(arr_type) = type_ {
                let list = vec
                    .iter()
                    .map(|val| parse_serde_value(val, arr_type))
                    .collect::<Result<_, _>>()?;
                Ok(Value::List(list))
            } else {
                Err(Error::InvalidJson)
            }
        }
        SerdeValue::Object(inner_json_obj) => {
            if let Type::Struct(struct_type) = type_ {
                Ok(Value::Struct(transform_serde_obj(
                    inner_json_obj,
                    struct_type,
                )?))
            } else {
                Err(Error::InvalidJson)
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
        let sier_value = parse_serde_value(
            json_obj.get(field_def.name()).ok_or(Error::InvalidJson)?,
            field_def.type_(),
        )?;
        values.push(sier_value);
    }

    Ok(Object::new(def.as_ref(), values))
}
