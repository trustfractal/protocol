//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use merklex_js;

static merke_empty: &str    = "4004786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce";
static merkle_a: &str       = "4004021ced8799296ceca557832ab941a50b4a11f83478cf141f51f933f653ab9fbcc05a037cddbed06e309bf334942c4e58cdf1a46e237911ccd7fcf9787cbc7fd0";
static merkle_b_ext_a: &str = "4008d6f80189753931698cbde8a0ede7f4f33131c546dfa2e10ca4b69c897c4b7430fcac17dababf2db20115a5a59240cd6b465dc368356cb01170cf11ad1fe14167021ced8799296ceca557832ab941a50b4a11f83478cf141f51f933f653ab9fbcc05a037cddbed06e309bf334942c4e58cdf1a46e237911ccd7fcf9787cbc7fd0";

#[wasm_bindgen_test]
fn build_an_empty_merkle_tree() {
    let r = merklex_js::build("").unwrap();
    assert_eq!(r, merkle_a);
}

#[wasm_bindgen_test]
fn build_a_new_merkle_tree() {
    let r = merklex_js::build("hello world").unwrap();
    assert_eq!(r, merkle_a);
}

#[wasm_bindgen_test]
fn extend_a_merkle_tree() {
    let r = merklex_js::extend(merkle_a, "hello world").unwrap();
    assert_eq!(r, merkle_b_ext_a);
}
