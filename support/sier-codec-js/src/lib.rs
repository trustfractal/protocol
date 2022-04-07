use sier_codec::Parser;
use wasm_bindgen::prelude::*;
//TODO(melatron): Create a JS Class that holds the file_defs inside and
// serialize/deserialize are methods of this class.
#[wasm_bindgen]
pub fn serialize(js_object: JsValue, file_defs: &str, struct_def: &str) -> js_sys::Uint8Array {
    if !js_object.is_object() {
        wasm_bindgen::throw_str("Provided argument is not a JS Object.");
    }

    let json = js_sys::JSON::stringify(&js_object).unwrap_or_else(|e| {
        wasm_bindgen::throw_val(e);
    });
    let json = json
        .as_string()
        .expect("Parsed as string from JSON::stringify");

    let mut parser = Parser::default();
    parser.add_file_defs(file_defs).unwrap_or_else(|e| {
        wasm_bindgen::throw_str(format!("Wrong file definitions provided {0}", e).as_str());
    });

    let def = parser.struct_def(struct_def).unwrap_or_else(|| {
        wasm_bindgen::throw_str("Wrong struct_def provided");
    });

    let obj = parser.json_str(json.as_str(), def).unwrap_or_else(|e| {
        wasm_bindgen::throw_str(format!("Wrong JSON provided - {0}", e).as_str());
    });

    js_sys::Uint8Array::from(&obj.serialize()[..])
}

#[wasm_bindgen]
pub fn deserialize(sier: js_sys::Uint8Array, file_defs: &str) -> JsValue {
    let mut parser = Parser::default();
    parser.add_file_defs(file_defs).unwrap_or_else(|e| {
        wasm_bindgen::throw_str(format!("Wrong file definitions provided {0}", e).as_str());
    });

    let obj = parser.parse(&sier.to_vec()).unwrap_or_else(|e| {
        wasm_bindgen::throw_str(format!("Parse failed - {0}", e).as_str());
    });

    let json = sier_codec::json::transform_sier_obj(&obj).unwrap_or_else(|e| {
        wasm_bindgen::throw_str(format!("Transforming sier object failed - {0}", e).as_str());
    });

    JsValue::from_serde(&json).unwrap_or_else(|e| {
        wasm_bindgen::throw_str(
            format!("Serde Value to Wasm JsValue parsing failed - {0}", e).as_str(),
        );
    })
}
