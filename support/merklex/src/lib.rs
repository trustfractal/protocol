#![cfg_attr(not(feature = "std"), no_std)]

use digest::Digest;
use generic_array::{typenum::consts::U64, ArrayLength, GenericArray};
use sp_std::{
    collections::vec_deque::VecDeque,
    prelude::{Box, Vec},
};

#[derive(Debug, Clone)]
pub struct MerkleTree<D: Digest> {
    hash: GenericArray<u8, D::OutputSize>,
    children: Option<(Box<Self>, Box<Self>)>,
}

impl<D: Digest> MerkleTree<D> {
    pub fn hash(&self) -> &GenericArray<u8, D::OutputSize> {
        &self.hash
    }

    pub fn leaf(hash: GenericArray<u8, D::OutputSize>) -> Self {
        MerkleTree {
            hash,
            children: None,
        }
    }

    pub fn leaf_bytes(bytes: &[u8]) -> Self {
        MerkleTree {
            hash: D::digest(bytes),
            children: None,
        }
    }

    fn merge(l: Self, r: Self) -> Self {
        let mut hasher = D::new();
        hasher.update(&l.hash);
        hasher.update(&r.hash);

        MerkleTree {
            hash: hasher.finalize(),
            children: Some((Box::new(l), Box::new(r))),
        }
    }

    pub fn children(&self) -> Option<(&Self, &Self)> {
        self.children
            .as_ref()
            .map(|(l, r)| (l.as_ref(), r.as_ref()))
    }

    pub fn push(self, value: impl AsRef<[u8]>) -> Self {
        let new_leaf = Self::leaf_bytes(value.as_ref());

        match self.children {
            _ if self.balanced() => Self::merge(self, new_leaf),
            Some((left, right)) => Self::merge(*left, right.push(value)),
            None => unreachable!(),
        }
    }

    pub fn weight(&self) -> usize {
        self.children
            .as_ref()
            .map(|(l, r)| l.weight() + r.weight() + 1)
            .unwrap_or(1)
    }

    pub fn balanced(&self) -> bool {
        self.children
            .as_ref()
            .map(|(l, r)| l.weight() == r.weight())
            .unwrap_or(true)
    }
}

impl<D: Digest<OutputSize = U64>> MerkleTree<D> {
    pub(crate) fn leaf64(hash: [u8; 64]) -> MerkleTree<D> {
        MerkleTree {
            hash: GenericArray::from_exact_iter(hash.iter().cloned()).unwrap(),
            children: None,
        }
    }
}

impl<D: Digest> PartialEq for MerkleTree<D> {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

pub fn build_tree<D: Digest, T: AsRef<[u8]>, I: IntoIterator<Item = T>>(
    items: I,
) -> Option<MerkleTree<D>> {
    let leaves = items
        .into_iter()
        .map(|item| MerkleTree::leaf_bytes(item.as_ref()))
        .collect::<VecDeque<_>>();
    build_from_layer::<D>(leaves)
}

fn build_from_layer<D: Digest>(mut leaves: VecDeque<MerkleTree<D>>) -> Option<MerkleTree<D>> {
    if leaves.len() == 0 {
        return None;
    }
    if leaves.len() == 1 {
        return leaves.pop_front();
    }

    let mut next_layer = VecDeque::with_capacity(leaves.len() / 2);
    loop {
        match (leaves.pop_front(), leaves.pop_front()) {
            (Some(l), Some(r)) => {
                next_layer.push_back(MerkleTree::merge(l, r));
            }
            (None, None) => break,
            (Some(n), None) => {
                next_layer.push_back(n);
            }
            (None, Some(_)) => unreachable!(),
        }
    }
    build_from_layer::<D>(next_layer)
}

#[cfg(test)]
mod tests {
    use super::*;

    use blake2::Blake2b;
    use hex_literal::hex;

    fn hello_world_hash() -> [u8; 64] {
        hex!(
            "
            021ced8799296ceca557832ab941a50b4a11f83478cf141f51f933f653ab9fbc
            c05a037cddbed06e309bf334942c4e58cdf1a46e237911ccd7fcf9787cbc7fd0
        "
        )
    }

    #[cfg(test)]
    mod build_tree {
        use super::*;

        #[test]
        fn empty_input() {
            let tree = build_tree::<Blake2b, &&[u8], _>(&[]);

            assert_eq!(tree, None);
        }

        #[test]
        fn single_item() {
            let tree = build_tree::<Blake2b, _, _>(&[&b"hello world"[..]]);

            let expected = MerkleTree::leaf64(hello_world_hash());

            assert_eq!(tree, Some(expected));
        }

        #[test]
        fn two_items() {
            let tree = build_tree::<Blake2b, _, _>(&[&b"hello world"[..], &b"hello world"[..]]);

            let mut hasher = Blake2b::new();
            hasher.update(&hello_world_hash());
            hasher.update(&hello_world_hash());
            let hash = hasher.finalize();

            assert_eq!(tree.unwrap().hash(), &hash);
        }

        #[test]
        fn two_items_children() {
            let tree =
                build_tree::<Blake2b, _, _>(&[&b"hello world"[..], &b"hello world"[..]]).unwrap();

            assert_eq!(
                tree.children(),
                Some((
                    &MerkleTree::leaf64(hello_world_hash()),
                    &MerkleTree::leaf64(hello_world_hash())
                ))
            );
        }

        #[test]
        fn three_items() {
            let tree = build_tree::<Blake2b, _, _>(&[
                &b"hello world"[..],
                &b"hello world"[..],
                &b"hello world"[..],
            ])
            .unwrap();

            let (left, right) = tree.children().unwrap();
            assert_eq!(right, &MerkleTree::leaf64(hello_world_hash()));

            assert_eq!(
                left.children(),
                Some((
                    &MerkleTree::leaf64(hello_world_hash()),
                    &MerkleTree::leaf64(hello_world_hash())
                ))
            );
        }
    }

    #[cfg(test)]
    mod push {
        use super::*;

        #[test]
        fn onto_leaf() {
            let leaf = MerkleTree::leaf64(hello_world_hash());
            let pushed = leaf.push(&b"hello world"[..]);

            let from_sequence =
                build_tree::<Blake2b, _, _>(&[&b"hello world"[..], &b"hello world"[..]]).unwrap();

            assert_eq!(pushed, from_sequence);
        }

        #[test]
        fn imbalanced_tree() {
            let three_items = build_tree::<Blake2b, _, _>(&[
                &b"hello world"[..],
                &b"hello world"[..],
                &b"hello world"[..],
            ])
            .unwrap();

            let four_items = build_tree::<Blake2b, _, _>(&[
                &b"hello world"[..],
                &b"hello world"[..],
                &b"hello world"[..],
                &b"hello world"[..],
            ])
            .unwrap();

            let pushed = three_items.push(&b"hello world"[..]);

            assert_eq!(pushed, four_items);
        }

        #[test]
        fn balanced_tree() {
            let two_items =
                build_tree::<Blake2b, _, _>(&[&b"hello world"[..], &b"hello world"[..]]).unwrap();

            let three_items = build_tree::<Blake2b, _, _>(&[
                &b"hello world"[..],
                &b"hello world"[..],
                &b"hello world"[..],
            ])
            .unwrap();

            let pushed = two_items.push(&b"hello world"[..]);

            assert_eq!(pushed, three_items);
        }
    }
}
