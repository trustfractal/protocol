use crate::Hash;

use blake2::{Blake2b512, Digest};

pub(crate) struct MerkleTree {
    hashes: Vec<(u8, Hash)>,
}

impl MerkleTree {
    pub fn new() -> Self {
        MerkleTree { hashes: Vec::new() }
    }

    pub fn update(&mut self, hash: Hash) {
        self.hashes.push((0, hash));
        self.collapse();
    }

    fn collapse(&mut self) {
        loop {
            match self.hashes[..] {
                [] => unreachable!(),
                [_] => return,
                [.., (w0, _), (w1, _)] if w0 != w1 => return,
                _ => {
                    let (w, right) = self.hashes.pop().unwrap();
                    let (_, left) = self.hashes.pop().unwrap();

                    self.hashes.push((w + 1, digest_pair(left, right)));
                }
            }
        }
    }

    pub fn finalize(mut self) -> Hash {
        let mut current = match self.hashes.pop() {
            Some((_, h)) => h,
            None => [0; 64],
        };
        while let Some((_, h)) = self.hashes.pop() {
            current = digest_pair(h, current);
        }
        current
    }
}

fn digest_pair(h1: Hash, h2: Hash) -> Hash {
    let mut hasher = Blake2b512::new();

    hasher.update(h1);
    hasher.update(h2);

    Hash::from(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_empty_hash() {
        let tree = MerkleTree::new();

        assert_eq!(tree.finalize(), [0; 64]);
    }

    #[test]
    fn single_hash_is_hash() {
        let mut tree = MerkleTree::new();

        tree.update([0; 64]);

        assert_eq!(tree.finalize(), [0; 64]);
    }

    #[test]
    fn two_values_is_concatenation() {
        let mut tree = MerkleTree::new();

        tree.update([0; 64]);
        tree.update([1; 64]);

        assert_eq!(tree.finalize(), digest_pair([0; 64], [1; 64]));
    }

    #[test]
    fn four_values_is_combination_of_combinations() {
        let mut tree = MerkleTree::new();

        tree.update([0; 64]);
        tree.update([1; 64]);
        tree.update([2; 64]);
        tree.update([3; 64]);

        let left = digest_pair([0; 64], [1; 64]);
        let right = digest_pair([2; 64], [3; 64]);
        let root = digest_pair(left, right);

        assert_eq!(tree.finalize(), root);
    }

    #[test]
    fn three_values() {
        let mut tree = MerkleTree::new();

        tree.update([0; 64]);
        tree.update([1; 64]);
        tree.update([2; 64]);

        let left = digest_pair([0; 64], [1; 64]);
        let right = [2; 64];
        let root = digest_pair(left, right);

        assert_eq!(tree.finalize(), root);
    }
}
