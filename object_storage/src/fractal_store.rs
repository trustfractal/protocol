use crate::{merkle_tree::MerkleTree, *};

use fallible_iterator::FallibleIterator;
use std::ops::Deref;

#[mockall_double::double]
use crate::kv_set::KvSet;

pub struct FractalStore<D> {
    db: Handle<D>,
    objects: KvSet,
}

impl<D: Database + 'static> FractalStore<D> {
    pub fn new(db: D) -> Self {
        Self::new_with_deps(db, KvSet::new(&[2]))
    }

    fn new_with_deps(db: D, objects: KvSet) -> Self {
        FractalStore {
            db: Handle::new(db),
            objects,
        }
    }

    pub fn init(&mut self, id: &[u8], value: &[u8]) -> Result<(), Error<D::Error>> {
        if self.objects.contains(id, &*self.db.borrow())? {
            return Err(Error::IdExists);
        }

        let mut borrow = self.db.borrow_mut();
        borrow.store_iter([1].iter().chain(id), value)?;
        self.objects.insert(id, &mut *borrow)?;

        Ok(())
    }

    pub fn root_hash(&self) -> Result<Hash, Error<D::Error>> {
        let mut merkle_tree = MerkleTree::new();

        let mut object_ids = self.objects.iter_lexicographic(self.db.clone());
        while let Some(object_id) = object_ids.next()? {
            merkle_tree.update(self.object_hash(object_id.as_ref())?);
        }

        Ok(merkle_tree.root_hash())
    }

    fn object_hash(&self, _object_id: &[u8]) -> Result<Hash, Error<D::Error>> {
        unimplemented!("object_hash");
    }

    pub fn prove_given(&self, _given: Given, _prop: Proposition) -> Result<Proof, D::Error> {
        unimplemented!("prove_given");
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum Error<E> {
    Db(#[from] E),
    IdExists,
}

pub trait Database {
    type Error;

    fn store(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;

    fn store_iter<'b>(
        &mut self,
        key: impl IntoIterator<Item = &'b u8>,
        value: &[u8],
    ) -> Result<(), Self::Error> {
        self.store(key.into_iter().cloned().collect::<Vec<_>>().as_ref(), value)
    }

    fn store_slices(&mut self, key_slices: &[&[u8]], value: &[u8]) -> Result<(), Self::Error> {
        self.store_iter(key_slices.iter().flat_map(|v| v.iter()), value)
    }

    fn read_iter<'b>(
        &self,
        key: impl IntoIterator<Item = &'b u8>,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        self.read(key.into_iter().cloned().collect::<Vec<_>>().as_ref())
    }

    fn read_slices(&self, key_slices: &[&[u8]]) -> Result<Option<Vec<u8>>, Self::Error> {
        self.read_iter(key_slices.iter().flat_map(|v| v.iter()))
    }
}

pub struct Handle<T>(std::rc::Rc<std::cell::RefCell<T>>);

impl<T> Handle<T> {
    pub fn new(item: T) -> Self {
        Handle(std::rc::Rc::new(std::cell::RefCell::new(item)))
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle(std::rc::Rc::clone(&self.0))
    }
}

impl<T> Deref for Handle<T> {
    type Target = std::cell::RefCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::kv_set::MockKvSet;
    use mockall::predicate::*;
    use std::{cell::RefCell, rc::Rc};

    fn mock_kv_set(expectations: impl FnOnce(&mut MockKvSet)) -> MockKvSet {
        let mut kv_set = MockKvSet::default();
        expectations(&mut kv_set);
        kv_set.expect_insert::<Rc<_>>().return_const(Ok(()));
        kv_set.expect_contains::<Rc<_>>().return_const(Ok(false));
        kv_set
    }

    #[cfg(test)]
    mod init {
        use super::*;

        #[test]
        fn stores_bytes_in_value() {
            let db = Rc::new(RefCell::new(crate::test::InMemoryDb::default()));
            let mut store = FractalStore::new_with_deps(db.clone(), mock_kv_set(|_| {}));

            store.init(&[42], b"foo").unwrap();

            assert_eq!(db.borrow().read(&[1, 42]).unwrap(), Some(b"foo".to_vec()));
        }

        #[test]
        fn adds_item_id_to_kv_storage_set() {
            let kv_set = mock_kv_set(|kv_set| {
                kv_set
                    .expect_insert::<Rc<_>>()
                    .return_const(Ok(()))
                    .with(eq(&[42][..]), always())
                    .once();
            });

            let db = Rc::new(RefCell::new(crate::test::InMemoryDb::default()));
            let mut store = FractalStore::new_with_deps(db.clone(), kv_set);

            store.init(&[42], b"foo").unwrap();
        }

        #[test]
        fn fails_if_id_exists() {
            let kv_set = mock_kv_set(|kv_set| {
                kv_set
                    .expect_contains::<Rc<_>>()
                    .with(eq(&[42][..]), always())
                    .return_const(Ok(true));
            });

            let db = Rc::new(RefCell::new(crate::test::InMemoryDb::default()));
            let mut store = FractalStore::new_with_deps(db.clone(), kv_set);

            assert_eq!(store.init(&[42], b"foo"), Err(Error::IdExists));
            assert_eq!(db.borrow().read(&[1, 42]).unwrap(), None);
        }
    }
}
