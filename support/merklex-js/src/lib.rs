mod utils;

use blake2::Blake2b;
use merklex::MerkleTree;
use parity_scale_codec::{Decode, Encode};

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn build(s: &str) -> Option<String> {
    MerkleTree::<Blake2b>::from_iter(&[s]).map(|v| hex::encode(v.encode()))
}

#[wasm_bindgen]
pub fn extend(mtree: &str, s: &str) -> Option<String> {
    decode_hex_mtree(mtree).map(|tree| hex::encode(tree.push(s).encode()))
}

#[wasm_bindgen]
pub fn extend_multiple(mtree: &str, leaves: JsValue) -> Option<String> {
    let mut tree = decode_hex_mtree(mtree)?;

    let leaves: Vec<String> = leaves.into_serde().ok()?;
    for leave in leaves {
        tree = tree.push(leave);
    }

    Some(hex::encode(tree.encode()))
}

#[wasm_bindgen]
pub fn strict_extension_proof(mtree_a: &str, mtree_b: &str) -> Option<String> {
    let mtree_a = decode_hex_mtree(mtree_a)?;
    let mtree_b = decode_hex_mtree(mtree_b)?;

    let strict_proof = mtree_a.strict_extension_proof(&mtree_b)?;

    Some(hex::encode(strict_proof.encode()))
}

#[wasm_bindgen]
pub fn prune_balanced(tree: &str) -> Option<String> {
    let tree = decode_hex_mtree(tree)?;
    let pruned = tree.prune_balanced();
    Some(hex::encode(pruned.encode()))
}

fn decode_hex_mtree(mtree: &str) -> Option<MerkleTree<Blake2b>> {
    let buffer = hex::decode(mtree).ok()?;
    MerkleTree::<Blake2b>::decode(&mut buffer.as_ref()).ok()
}
