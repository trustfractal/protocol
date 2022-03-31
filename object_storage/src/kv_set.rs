use crate::*;

use fallible_iterator::FallibleIterator;
use std::cmp::Ordering;

pub(crate) struct KvSet<D> {
    handle: PrefixedHandle<D>,
}

type Path = Vec<bool>;

#[cfg_attr(test, mockall::automock, allow(dead_code))]
impl<D: Database + 'static> KvSet<D> {
    pub fn new(handle: PrefixedHandle<D>) -> Self {
        KvSet { handle }
    }

    pub fn insert(&mut self, key: &[u8]) -> Result<(), D::Error> {
        self.insert_at(key, Vec::new())?;
        Ok(())
    }

    pub fn insert_at(&mut self, key: &[u8], path: Path) -> Result<(), D::Error> {
        if let Find::Missing(path) = self.find_placement(path, key)? {
            let mut value = Vec::with_capacity(key.len() + 1);
            value.extend(key);
            value.push(0);
            self.handle.store_slices(&[&pack(&path)], &value)?;
        }

        Ok(())
    }

    fn find_placement(&self, mut path: Path, key: &[u8]) -> Result<Find, D::Error> {
        let bytes = match self.handle.read_slices(&[&pack(&path)])? {
            None => return Ok(Find::Missing(path)),
            Some(bytes) => bytes,
        };
        let existing_key = &bytes[..(bytes.len() - 1)];
        match lexicographic_compare(key, existing_key) {
            Ordering::Less => {
                path.push(false);
                self.find_placement(path, key)
            }
            Ordering::Greater => {
                path.push(true);
                self.find_placement(path, key)
            }
            Ordering::Equal => Ok(Find::Found(path, bytes)),
        }
    }

    pub fn contains(&self, key: &[u8]) -> Result<bool, D::Error> {
        Ok(match self.find_placement(Vec::new(), key)? {
            Find::Missing(_) => false,
            Find::Found(_, _) => true,
        })
    }

    // This takes a handle instead of a &D because mockall cannot mock methods with both generic
    // types and lifetimes. So we provide an owning ref.
    pub fn iter_lexicographic(&self) -> impl FallibleIterator<Item = Vec<u8>, Error = D::Error> {
        let handle = self.handle.clone();

        enum StackItem {
            AlreadyRead(Vec<u8>),
            DoRead(Path),
        }

        let mut path_stack = vec![StackItem::DoRead(vec![])];

        fallible_iterator::convert(std::iter::from_fn(move || loop {
            let path = match path_stack.pop()? {
                StackItem::AlreadyRead(bytes) => return Some(Ok(bytes)),
                StackItem::DoRead(path) => path,
            };

            let read = handle.read_slices(&[&pack(&path)]);

            let bytes = match read {
                Err(e) => return Some(Err(e)),
                Ok(v) => v,
            };

            if let Some(mut key) = bytes {
                key.pop();

                let mut left_path = path.clone();
                left_path.push(false);

                let mut right_path = path;
                right_path.push(true);

                // Actual operations are performed in reverse of this order.
                path_stack.push(StackItem::DoRead(right_path));
                path_stack.push(StackItem::AlreadyRead(key));
                path_stack.push(StackItem::DoRead(left_path));
            }
        }))
    }
}

fn pack(bools: &[bool]) -> Vec<u8> {
    let mut result = Vec::with_capacity(bools.len() / 7 + 1);

    for byte_worth in bools.chunks(8) {
        let mut byte = 0x80;
        for b in byte_worth.iter().rev() {
            byte >>= 1;
            if *b {
                byte |= 0x80;
            }
        }
        result.push(byte);
    }
    if bools.len() % 8 == 0 {
        result.push(0x80);
    }

    result
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

enum Find {
    // Node wasn't found, but would go here.
    Missing(Path),
    Found(Path, Vec<u8>),
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
    mod pack {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(pack(&[]), vec![0b1000_0000]);
        }

        #[test]
        fn one_false() {
            assert_eq!(pack(&[false]), vec![0b0100_0000]);
        }

        #[test]
        fn one_true() {
            assert_eq!(pack(&[true]), vec![0b1100_0000]);
        }

        #[test]
        fn eight_trues() {
            assert_eq!(pack(&vec![true; 8]), vec![0b1111_1111, 0b1000_0000]);
        }

        #[test]
        fn trues_then_falses() {
            let mut vec = vec![true; 8];
            vec.extend(vec![false; 8]);
            assert_eq!(pack(&vec), vec![0b1111_1111, 0b0000_0000, 0b1000_0000]);
        }

        #[test]
        fn full_byte_then_offset() {
            let mut vec = vec![true; 8];
            vec.extend(vec![false; 6]);
            assert_eq!(pack(&vec), vec![0b1111_1111, 0b0000_0010]);
        }

        #[test]
        fn alternating() {
            let mut vec = Vec::new();
            for _ in 0..3 {
                vec.push(true);
                vec.push(false);
            }
            vec.push(true);
            assert_eq!(pack(&vec), vec![0b10101011]);
        }
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
