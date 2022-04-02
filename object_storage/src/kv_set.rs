use crate::*;

use fallible_iterator::FallibleIterator;
use parity_scale_codec::{Compact, Decode, Encode};
use std::cmp::Ordering;

pub(crate) struct KvSet<D> {
    handle: PrefixedHandle<D>,
}

// TODO(shelbyd): Reduce number of reads / writes.
#[cfg_attr(test, mockall::automock, allow(dead_code))]
impl<D: Database + 'static> KvSet<D> {
    pub fn new(handle: PrefixedHandle<D>) -> Self {
        KvSet { handle }
    }

    pub fn insert(&mut self, key: &[u8]) -> Result<(), D::Error> {
        self.insert_at(0, key)?;
        Ok(())
    }

    fn get_next_index(&mut self) -> Result<u64, D::Error> {
        let read = self.handle.read(&[])?;
        let index = match read {
            Some(bytes) => <Compact<u64> as Decode>::decode(&mut &bytes[..]).unwrap().0,
            None => 1,
        };
        self.handle.store(&[], &Compact(index + 1).encode())?;
        Ok(index)
    }

    pub fn insert_at(&mut self, index: u64, key: &[u8]) -> Result<bool, D::Error> {
        let did_insert = self.do_insert(index, key)?;
        if did_insert {
            self.rebalance(index)?;
        }
        Ok(did_insert)
    }

    fn do_insert(&mut self, index: u64, key: &[u8]) -> Result<bool, D::Error> {
        let make_node = || Node {
            left: None,
            right: None,
            height: 1,
            data: key.to_vec(),
        };

        let read = self.get_node(index)?;
        let mut node = match read {
            None => {
                self.set_node(index, make_node())?;
                return Ok(true);
            }
            Some(node) => node,
        };
        let dest = match lexicographic_compare(key, &node.data) {
            Ordering::Less => &mut node.left,
            Ordering::Greater => &mut node.right,
            Ordering::Equal => return Ok(false),
        };
        if let Some(i) = dest {
            let inserted = self.insert_at(*i, key)?;
            self.set_node(index, node)?;
            return Ok(inserted);
        }

        let new_index = self.get_next_index()?;
        *dest = Some(new_index);

        self.set_node(new_index, make_node())?;
        self.set_node(index, node)?;

        Ok(true)
    }

    fn set_node(&mut self, index: u64, mut node: Node) -> Result<(), D::Error> {
        let left_height = match node.left {
            None => 0,
            Some(l) => self.get_node(l)?.unwrap().height,
        };
        let right_height = match node.right {
            None => 0,
            Some(r) => self.get_node(r)?.unwrap().height,
        };

        node.height = core::cmp::max(left_height, right_height) + 1;
        self.handle.store(&Compact(index).encode(), &node.encode())
    }

    fn rebalance(&mut self, index: u64) -> Result<(), D::Error> {
        let this = self.get_node(index)?.unwrap();

        match (this.left, this.right) {
            (None, None) => return Ok(()),
            (None, Some(i)) => {
                let right = self.get_node(i)?.unwrap();
                if right.height == 0 {
                    self.set_node(index, this)?;
                    return Ok(());
                }

                self.rotate_right(i)?;
                self.rotate_left(index)?;
                Ok(())
            }
            (Some(i), None) => {
                let left = self.get_node(i)?.unwrap();
                if left.height == 0 {
                    self.set_node(index, this)?;
                    return Ok(());
                }

                self.rotate_left(i)?;
                self.rotate_right(index)?;
                Ok(())
            }
            (Some(l), Some(r)) => {
                let left = self.get_node(l)?.unwrap();
                let right = self.get_node(r)?.unwrap();

                match right.height.wrapping_sub(left.height) {
                    255 | 0 | 1 => Ok(()),
                    2 => self.rotate_left(index),
                    254 => self.rotate_right(index),
                    i => unreachable!("{:?}", i),
                }
            }
        }
    }

    fn rotate_right(&mut self, index: u64) -> Result<(), D::Error> {
        let mut this = self.get_node(index)?.unwrap();
        let (l_index, mut left) = match this.left {
            None => return Ok(()),
            Some(l) => (l, self.get_node(l)?.unwrap()),
        };
        // This and left swap indices.
        this.left = left.right;
        left.right = Some(l_index);

        self.set_node(l_index, this)?;
        self.set_node(index, left)?;

        Ok(())
    }

    fn rotate_left(&mut self, index: u64) -> Result<(), D::Error> {
        let mut this = self.get_node(index)?.unwrap();
        let (r_index, mut right) = match this.right {
            None => return Ok(()),
            Some(r) => (r, self.get_node(r)?.unwrap()),
        };
        // This and left swap indices.
        this.right = right.left;
        right.left = Some(r_index);

        self.set_node(r_index, this)?;
        self.set_node(index, right)?;

        Ok(())
    }

    pub fn contains(&self, key: &[u8]) -> Result<bool, D::Error> {
        let node = self.get_node(0)?;
        Ok(match node {
            None => false,
            Some(n) => n.data == key,
        })
    }

    fn get_node(&self, index: u64) -> Result<Option<Node>, D::Error> {
        Ok(self
            .handle
            .read(&Compact(index).encode())?
            .map(Node::decode))
    }

    pub fn iter_lexicographic(&self) -> impl FallibleIterator<Item = Vec<u8>, Error = D::Error> {
        let handle = self.handle.clone();

        enum StackItem {
            AlreadyRead(Vec<u8>),
            DoRead(u64),
        }

        let mut stack = vec![StackItem::DoRead(0)];

        fallible_iterator::convert(std::iter::from_fn(move || loop {
            let index = match stack.pop()? {
                StackItem::AlreadyRead(bytes) => return Some(Ok(bytes)),
                StackItem::DoRead(index) => index,
            };

            let bytes = match handle.read(&Compact(index).encode()) {
                Err(e) => return Some(Err(e)),
                Ok(v) => v,
            };

            if let Some(key) = bytes {
                let node = Node::decode(key);

                // Actual operations are performed in reverse of this order.
                if let Some(r) = node.right {
                    stack.push(StackItem::DoRead(r));
                }
                stack.push(StackItem::AlreadyRead(node.data));
                if let Some(l) = node.left {
                    stack.push(StackItem::DoRead(l));
                }
            }
        }))
    }

    #[cfg(test)]
    fn height(&self, index: u64) -> Result<u8, D::Error> {
        let node = match self.get_node(index)? {
            None => return Ok(0),
            Some(n) => n,
        };

        Ok(1 + match (node.left, node.right) {
            (None, None) => 0,
            (Some(n), None) | (None, Some(n)) => self.height(n)?,
            (Some(l), Some(r)) => core::cmp::max(self.height(l)?, self.height(r)?),
        })
    }
}

#[derive(Debug)]
struct Node {
    left: Option<u64>,
    right: Option<u64>,
    height: u8,
    data: Vec<u8>,
}

impl Node {
    fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.left.map(Compact).encode_to(&mut out);
        self.right.map(Compact).encode_to(&mut out);
        out.push(self.height);
        out.extend(&self.data);
        out
    }

    fn decode(mut bytes: Vec<u8>) -> Self {
        let mut slice = &bytes[..];
        let left = <Option<Compact<u64>> as Decode>::decode(&mut slice)
            .unwrap()
            .map(|c| c.0);
        let right = <Option<Compact<u64>> as Decode>::decode(&mut slice)
            .unwrap()
            .map(|c| c.0);
        let (&height, slice) = slice.split_first().unwrap();

        let index = bytes.len() - slice.len();
        let data = bytes.split_off(index);
        Node {
            left,
            right,
            height,
            data,
        }
    }
}

fn lexicographic_compare(mut a: &[u8], mut b: &[u8]) -> Ordering {
    loop {
        return match (a.get(0), b.get(0)) {
            (None, None) => Ordering::Equal,
            (None, _) => Ordering::Less,
            (_, None) => Ordering::Greater,
            (Some(a0), Some(b0)) => match a0.cmp(b0) {
                Ordering::Equal => {
                    a = &a[1..];
                    b = &b[1..];
                    continue;
                }
                o => o,
            },
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test::*;

    #[test]
    fn new_contains_nothing() {
        let db = Handle::new(InMemoryDb::default());
        let kv_set = KvSet::new(PrefixedHandle::new(&[1, 2, 3], &db));

        assert_eq!(kv_set.contains(b"foo"), Ok(false));
    }

    #[test]
    fn insert_contains_key() {
        let db = Handle::new(InMemoryDb::default());
        let mut kv_set = KvSet::new(PrefixedHandle::new(&[1, 2, 3], &db));

        kv_set.insert(b"foo").unwrap();

        assert_eq!(kv_set.contains(b"foo"), Ok(true));
    }

    #[test]
    fn prefixes_db_entries_with_provided_prefix() {
        let db = Handle::new(InMemoryDb::default());
        let prefix = &[1, 2, 3];
        let mut kv_set = KvSet::new(PrefixedHandle::new(prefix, &db));

        kv_set.insert(b"foo").unwrap();

        for key in db.borrow().keys() {
            assert_eq!(&key[..prefix.len()], prefix);
        }
    }

    #[cfg(test)]
    mod iter_lexicographic {
        use super::*;

        #[test]
        fn empty_is_empty_iter() {
            let db = Handle::new(InMemoryDb::default());
            let kv_set = KvSet::new(PrefixedHandle::new(&[1, 2, 3], &db));

            assert_eq!(kv_set.iter_lexicographic().next(), Ok(None));
        }

        #[test]
        fn contains_a_single_value() {
            let db = Handle::new(InMemoryDb::default());
            let mut kv_set = KvSet::new(PrefixedHandle::new(&[1, 2, 3], &db));

            kv_set.insert(&[42]).unwrap();

            let mut iter = kv_set.iter_lexicographic();
            assert_eq!(iter.next(), Ok(Some(vec![42])));
            assert_eq!(iter.next(), Ok(None));
        }

        #[test]
        fn contains_multiple_values() {
            let db = Handle::new(InMemoryDb::default());
            let mut kv_set = KvSet::new(PrefixedHandle::new(&[1, 2, 3], &db));

            kv_set.insert(&[42]).unwrap();
            kv_set.insert(&[43]).unwrap();
            kv_set.insert(&[44]).unwrap();

            let mut iter = kv_set.iter_lexicographic();
            assert_eq!(iter.next(), Ok(Some(vec![42])));
            assert_eq!(iter.next(), Ok(Some(vec![43])));
            assert_eq!(iter.next(), Ok(Some(vec![44])));
            assert_eq!(iter.next(), Ok(None));
        }

        #[test]
        fn returns_values_in_lexicographic_order() {
            let db = Handle::new(InMemoryDb::default());
            let mut kv_set = KvSet::new(PrefixedHandle::new(&[1, 2, 3], &db));

            kv_set.insert(&[44]).unwrap();
            kv_set.insert(&[43]).unwrap();
            kv_set.insert(&[42]).unwrap();

            let mut iter = kv_set.iter_lexicographic();
            assert_eq!(iter.next(), Ok(Some(vec![42])));
            assert_eq!(iter.next(), Ok(Some(vec![43])));
            assert_eq!(iter.next(), Ok(Some(vec![44])));
            assert_eq!(iter.next(), Ok(None));
        }
    }

    #[cfg(test)]
    mod rebalancing {
        use super::*;

        #[test]
        fn simple() {
            let db = Handle::new(InMemoryDb::default());
            let mut kv_set = KvSet::new(PrefixedHandle::new(&[1, 2, 3], &db));

            assert_eq!(kv_set.height(0).unwrap(), 0);

            kv_set.insert(&[42]).unwrap();
            assert_eq!(kv_set.height(0).unwrap(), 1);

            kv_set.insert(&[44]).unwrap();
            kv_set.insert(&[43]).unwrap();

            assert_eq!(kv_set.height(0).unwrap(), 2);
        }

        #[test]
        fn left_heavy() {
            let db = Handle::new(InMemoryDb::default());
            let mut kv_set = KvSet::new(PrefixedHandle::new(&[1, 2, 3], &db));

            for i in (0..255).rev() {
                kv_set.insert(&[i]).unwrap();
            }

            assert_eq!(kv_set.height(0).unwrap(), 8);
        }

        #[test]
        fn right_heavy() {
            let db = Handle::new(InMemoryDb::default());
            let mut kv_set = KvSet::new(PrefixedHandle::new(&[1, 2, 3], &db));

            for i in 0..255 {
                kv_set.insert(&[i]).unwrap();
            }

            assert_eq!(kv_set.height(0).unwrap(), 8);
        }

        // TODO(shelbyd): Quickcheck tests for balancing.
    }

    #[test]
    fn lexicographic_compare_test() {
        assert_eq!(lexicographic_compare(&[], &[]), Ordering::Equal);
        assert_eq!(lexicographic_compare(&[0], &[]), Ordering::Greater);
        assert_eq!(lexicographic_compare(&[], &[0]), Ordering::Less);

        assert_eq!(lexicographic_compare(&[0], &[0]), Ordering::Equal);
        assert_eq!(lexicographic_compare(&[1], &[0]), Ordering::Greater);
        assert_eq!(lexicographic_compare(&[0], &[1]), Ordering::Less);

        assert_eq!(lexicographic_compare(&[0, 0], &[0, 0]), Ordering::Equal);
        assert_eq!(lexicographic_compare(&[0, 1], &[0, 0]), Ordering::Greater);
        assert_eq!(lexicographic_compare(&[0, 0], &[0, 1]), Ordering::Less);
    }
}
