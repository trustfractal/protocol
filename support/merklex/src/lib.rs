#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[macro_use]
extern crate quickcheck_macros;

use digest::Digest;
use generic_array::{typenum::consts::U64, GenericArray};
use sp_std::{collections::vec_deque::VecDeque, prelude::Box};

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

    pub fn leaf(hash: GenericArray<u8, D::OutputSize>) -> Self {
        MerkleTree {
            hash,
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
        match &self.children {
            None => true,
            Some((l, r)) => {
                debug_assert!(l.balanced());
                l.max_depth() == r.max_depth() && r.balanced()
            }
        }
    }

    fn max_depth(&self) -> usize {
        match &self.children {
            None => 0,
            Some((l, _)) => l.max_depth() + 1,
        }
    }

    pub fn extends(&self, other: &Self) -> bool {
        match (self.children(), other.children()) {
            _ if self == other => true,
            (Some((self_l, _)), _) if self_l.extends(other) => true,
            (Some((self_l, self_r)), Some((other_l, other_r))) if self_l == other_l => {
                self_r.extends(other_r)
            }
            _ => false,
        }
    }

    pub fn extension_proof(&self, other: &Self) -> Option<Self> {
        match (self.children(), other.children()) {
            _ if self == other && self.balanced() => Some(Self::leaf(self.hash.clone())),
            (Some((self_l, self_r)), Some((other_l, other_r))) if self_l == other_l => {
                let left = Self::leaf(self_l.hash.clone());
                let right = self_r.extension_proof(other_r)?;
                Some(Self::merge(left, right))
            }
            (Some((self_l, self_r)), _) if self_l.extends(other) => {
                let left = self_l.extension_proof(other)?;
                let mut right = self_r.clone();
                right.prune_balanced();
                Some(Self::merge(left, right))
            }
            _ => None,
        }
    }

    fn prune_balanced(&mut self) {
        if self.balanced() {
            self.children = None;
            return;
        }

        if let Some((l, r)) = &mut self.children {
            l.prune_balanced();
            r.prune_balanced();
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

impl<D: Digest> Clone for MerkleTree<D> {
    fn clone(&self) -> Self {
        MerkleTree {
            hash: self.hash.clone(),
            children: self.children.clone(),
        }
    }
}

impl<D: Digest> core::fmt::Debug for MerkleTree<D> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        use core::fmt::Write;

        let hash_hex = {
            let mut s = String::new();
            for byte in &self.hash[..8] {
                write!(s, "{:02x}", byte)?;
            }
            s
        };

        f.debug_struct("MerkleTree")
            .field("hash", &hash_hex)
            .field("children", &self.children)
            .finish()
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

    #[cfg(test)]
    mod extension_proof {
        use super::*;

        #[test]
        fn same_tree() {
            let tree = MerkleTree::<Blake2b>::from_iter(&["", ""]).unwrap();
            let extension = tree.extension_proof(&tree);

            assert!(extension.is_some());
            assert!(extension.unwrap().extends(&tree));
        }

        #[test]
        fn simple_extension() {
            let tree = MerkleTree::<Blake2b>::from_iter(&["", ""]).unwrap();
            let extended = MerkleTree::<Blake2b>::from_iter(&["", "", ""]).unwrap();
            let extension = extended.extension_proof(&tree);

            assert!(extension.is_some());
            assert!(extension.unwrap().extends(&tree));
        }

        #[test]
        fn double_extension() {
            let tree = MerkleTree::<Blake2b>::from_iter(&["", "", ""]).unwrap();
            let extended = MerkleTree::<Blake2b>::from_iter(&["", "", "", ""]).unwrap();

            let extension = tree.extension_proof(&tree).unwrap();
            let new_extension = extended.extension_proof(&tree).unwrap();

            assert!(new_extension.extends(&extension));
            assert_eq!(extension.weight(), 3);
            assert_eq!(new_extension.weight(), 5);
        }

        #[test]
        fn chained_tree() {
            let first = MerkleTree::<Blake2b>::from_iter(&["", "", "", ""]).unwrap();
            let second = MerkleTree::<Blake2b>::from_iter(&["", "", "", "", "", "", ""]).unwrap();
            let third =
                MerkleTree::<Blake2b>::from_iter(&["", "", "", "", "", "", "", ""]).unwrap();

            let extension = second.extension_proof(&first).unwrap();
            let second_extension = third.extension_proof(&second).unwrap();

            assert!(second_extension.extends(&extension));
        }

        #[quickcheck]
        fn valid_extension(first: Vec<String>, second: Vec<String>) -> TestResult {
            if first.len() == 0 {
                return TestResult::discard();
            }

            let first_tree = MerkleTree::<Blake2b>::from_iter(first.clone()).unwrap();
            let second_tree =
                MerkleTree::<Blake2b>::from_iter(first.into_iter().chain(second)).unwrap();
            let extension = second_tree.extension_proof(&first_tree);

            TestResult::from_bool(extension.is_some() && extension.unwrap().extends(&first_tree))
        }

        #[quickcheck]
        fn item_changed(mut items: Vec<String>, index: usize) -> TestResult {
            if items.len() == 0 || index >= items.len() {
                return TestResult::discard();
            }

            let initial_tree = MerkleTree::<Blake2b>::from_iter(items.clone()).unwrap();

            items[index].push_str("something");
            let mutated_tree = MerkleTree::<Blake2b>::from_iter(items).unwrap();

            let extension = mutated_tree.extension_proof(&initial_tree);

            TestResult::from_bool(extension.is_none())
        }

        #[quickcheck]
        fn second_extension(
            first: Vec<String>,
            second: Vec<String>,
            third: Vec<String>,
        ) -> TestResult {
            if first.len() == 0 {
                return TestResult::discard();
            }

            let first_tree = MerkleTree::<Blake2b>::from_iter(first.iter()).unwrap();
            let second_tree =
                MerkleTree::<Blake2b>::from_iter(first.iter().chain(second.iter())).unwrap();
            let third_tree = MerkleTree::<Blake2b>::from_iter(
                first.iter().chain(second.iter()).chain(third.iter()),
            )
            .unwrap();

            let first_extension = second_tree.extension_proof(&first_tree).unwrap();
            let second_extension = third_tree.extension_proof(&second_tree).unwrap();

            TestResult::from_bool(second_extension.extends(&first_extension))
        }

        #[quickcheck]
        fn all_entries_the_same(
            counts: Vec<u8>,
        ) -> TestResult {
            if counts.len() == 0 || counts[0] == 0 || counts.len() > 5 {
                return TestResult::discard();
            }

            let totals = counts.into_iter()
                .scan(0usize, |total, count| {
                    *total += count as usize;
                    Some(*total)
                });


            let mut trees = Vec::new();

            for total in totals {
                let new_tree = MerkleTree::<Blake2b>::from_iter(vec![""; total]).unwrap();
                for tree in &trees {
                    let extension = new_tree.extension_proof(&tree).unwrap();
                    assert!(extension.extends(&tree));
                }
                trees.push(new_tree);
            }

            TestResult::passed()
        }
    }
}
