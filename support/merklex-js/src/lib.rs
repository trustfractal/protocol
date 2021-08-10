mod utils;

use blake2::Blake2b;
use merklex::MerkleTree;
use parity_scale_codec::Encode;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn build(s: &str) -> Option<Vec<u8>> {
    let tree = MerkleTree::<Blake2b>::from_iter(&[s])?;
    Some(tree.encode())
}
