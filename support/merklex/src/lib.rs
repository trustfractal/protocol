#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[macro_use]
extern crate quickcheck_macros;

use digest::Digest;
use generic_array::{typenum::consts::U64, GenericArray};
use sp_std::{collections::vec_deque::VecDeque, prelude::Box};

#[derive(Debug, Clone)]
pub struct MerkleTree<D: Digest> {
    hash: GenericArray<u8, D::OutputSize>,
    children: Option<(Box<Self>, Box<Self>)>,
}

impl<D: Digest> MerkleTree<D> {
    pub fn hash(&self) -> &GenericArray<u8, D::OutputSize> {
        &self.hash
    }

    pub fn from_iter<I: IntoIterator<Item = R>, R: AsRef<[u8]>>(items: I) -> Option<Self> {
        let leaves = items
            .into_iter()
            .map(|item| MerkleTree::leaf_bytes(item.as_ref()))
            .collect::<VecDeque<_>>();
        build_from_layer::<D>(leaves)
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

    pub fn extends(&self, other: &Self) -> bool {
        match &other.children {
            None => self.left_contains(other),
            Some((l, r)) => match self.sibling_of(l) {
                None => false,
                Some(sib) => sib.extends(r),
            },
        }
    }

    fn left(&self) -> Option<&Self> {
        self.children.as_ref().map(|(l, _)| l.as_ref())
    }

    fn left_contains(&self, other: &Self) -> bool {
        self == other || self.left().map(|l| l.left_contains(other)).unwrap_or(false)
    }

    // Only searches left-subtrees for other.
    fn sibling_of(&self, other: &Self) -> Option<&Self> {
        match &self.children {
            None => None,
            Some((l, r)) if l.as_ref() == other => Some(r.as_ref()),
            Some((l, _)) => l.sibling_of(other),
        }
    }
}

impl<D: Digest<OutputSize = U64>> MerkleTree<D> {
    #[cfg(test)]
    pub fn leaf64(hash: [u8; 64]) -> MerkleTree<D> {
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
    use quickcheck::TestResult;

    fn hello_world_hash() -> [u8; 64] {
        hex!(
            "
            021ced8799296ceca557832ab941a50b4a11f83478cf141f51f933f653ab9fbc
            c05a037cddbed06e309bf334942c4e58cdf1a46e237911ccd7fcf9787cbc7fd0
        "
        )
    }

    #[cfg(test)]
    mod building {
        use super::*;

        #[test]
        fn empty_input() {
            let tree = MerkleTree::<Blake2b>::from_iter(&[] as &[&[u8]]);

            assert_eq!(tree, None);
        }

        #[test]
        fn single_item() {
            let tree = MerkleTree::<Blake2b>::from_iter(&["hello world"]);

            let expected = MerkleTree::leaf64(hello_world_hash());

            assert_eq!(tree, Some(expected));
        }

        #[test]
        fn two_items() {
            let tree = MerkleTree::<Blake2b>::from_iter(&["hello world", "hello world"]);

            let mut hasher = Blake2b::new();
            hasher.update(&hello_world_hash());
            hasher.update(&hello_world_hash());
            let hash = hasher.finalize();

            assert_eq!(tree.unwrap().hash(), &hash);
        }

        #[test]
        fn two_items_children() {
            let tree = MerkleTree::<Blake2b>::from_iter(&["hello world", "hello world"]).unwrap();

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
            let tree =
                MerkleTree::<Blake2b>::from_iter(&["hello world", "hello world", "hello world"])
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
            let pushed = leaf.push("hello world");

            let from_sequence =
                MerkleTree::<Blake2b>::from_iter(&["hello world", "hello world"]).unwrap();

            assert_eq!(pushed, from_sequence);
        }

        #[test]
        fn imbalanced_tree() {
            let three_items =
                MerkleTree::<Blake2b>::from_iter(&["hello world", "hello world", "hello world"])
                    .unwrap();

            let four_items = MerkleTree::<Blake2b>::from_iter(&[
                "hello world",
                "hello world",
                "hello world",
                "hello world",
            ])
            .unwrap();

            let pushed = three_items.push("hello world");

            assert_eq!(pushed, four_items);
        }

        #[test]
        fn balanced_tree() {
            let two_items =
                MerkleTree::<Blake2b>::from_iter(&["hello world", "hello world"]).unwrap();

            let three_items =
                MerkleTree::<Blake2b>::from_iter(&["hello world", "hello world", "hello world"])
                    .unwrap();

            let pushed = two_items.push("hello world");

            assert_eq!(pushed, three_items);
        }
    }

    #[cfg(test)]
    mod extensions {
        use super::*;

        use quickcheck::TestResult;

        #[quickcheck]
        fn valid_extension(first: Vec<String>, second: Vec<String>) -> TestResult {
            if first.len() == 0 {
                return TestResult::discard();
            }

            let first_tree = MerkleTree::<Blake2b>::from_iter(first.clone()).unwrap();
            let second_tree =
                MerkleTree::<Blake2b>::from_iter(first.into_iter().chain(second)).unwrap();

            TestResult::from_bool(second_tree.extends(&first_tree))
        }

        #[quickcheck]
        fn item_changed(mut items: Vec<String>, index: usize) -> TestResult {
            if items.len() == 0 || index >= items.len() {
                return TestResult::discard();
            }

            let initial_tree = MerkleTree::<Blake2b>::from_iter(items.clone()).unwrap();

            items[index].push_str("something");
            let mutated_tree = MerkleTree::<Blake2b>::from_iter(items).unwrap();

            TestResult::from_bool(!mutated_tree.extends(&initial_tree))
        }
    }
}
