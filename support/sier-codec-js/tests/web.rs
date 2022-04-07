//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

use wasm_bindgen::prelude::*;

use sier_codec_js;

const JSON_STRUCT_DEF: &'static str = r#"
struct Corge {
    gz :u64;
    op :bool;
}

struct Foo {
    bar :u64;
    baz :string;
    qux :List<u64>;
    corge :Corge;
}
"#;

const JSON: &'static str = r#"
{
    "bar": 42,
    "baz": "abc",
    "qux": [4, 2],
    "corge": {
        "gz": 42,
        "op": true
    }
}
"#;

#[wasm_bindgen_test]
fn deserialize() {
    let json = js_sys::JSON::parse(JSON).unwrap();
    let sier = sier_codec_js::serialize(json, JSON_STRUCT_DEF, "Foo");
    let deserialized_json = sier_codec_js::deserialize(sier, JSON_STRUCT_DEF);
    let str_json = js_sys::JSON::stringify(&deserialized_json).unwrap();
    let left: String = str_json.into();

    assert_eq!(
        left,
        "{\"bar\":42,\"baz\":\"abc\",\"corge\":{\"gz\":42,\"op\":true},\"qux\":[4,2]}"
    );
}

#[wasm_bindgen_test]
fn serialize() {
    let json = js_sys::JSON::parse(JSON).unwrap();
    let sier = sier_codec_js::serialize(json, JSON_STRUCT_DEF, "Foo");

    assert_eq!(
        sier.to_vec(),
        vec![
            206, 206, 22, 230, 245, 223, 67, 43, 42, 0, 0, 0, 0, 0, 0, 0, 3, 97, 98, 99, 16, 4, 0,
            0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 115, 196, 212, 210, 235, 7, 68, 87, 42, 0, 0,
            0, 0, 0, 0, 0, 1
        ]
    );
}
