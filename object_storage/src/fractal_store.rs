use crate::{merkle_tree::MerkleTree, *};

use fallible_iterator::FallibleIterator;
use std::ops::Deref;

#[mockall_double::double]
use crate::kv_set::KvSet;

const OBJECT_DATA: &[u8] = &[1];
const OBJECT_IDS: &[u8] = &[2];

pub struct FractalStore<D: Database + 'static> {
    db: Handle<D>,
    objects: KvSet<D>,
}

impl<D: Database + 'static> FractalStore<D> {
    pub fn new(db: D) -> Self {
        let handle = Handle::new(db);
        let kv = KvSet::new(PrefixedHandle::new(OBJECT_IDS, &handle));
        Self::new_with_deps(handle, kv)
    }

    fn new_with_deps(db: Handle<D>, objects: KvSet<D>) -> Self {
        FractalStore { db, objects }
    }

    pub fn init(&mut self, id: &[u8], value: &[u8]) -> Result<(), InitError<D::Error>> {
        if self.objects.contains(id)? {
            return Err(InitError::IdExists);
        }

        self.db
            .borrow_mut()
            .store_slices(&[OBJECT_DATA, id], value)?;
        self.objects.insert(id)?;

        Ok(())
    }

    pub fn root_hash(&self) -> Result<Hash, D::Error> {
        let mut merkle_tree = MerkleTree::new();

        let mut object_ids = self.objects.iter_lexicographic();
        while let Some(object_id) = object_ids.next()? {
            let data = self.object_data(&object_id)?.unwrap();
            merkle_tree.update(object_hash(&object_id, &data));
        }

        Ok(merkle_tree.finalize())
    }

    fn object_data(&self, object_id: &[u8]) -> Result<Option<Vec<u8>>, D::Error> {
        self.db.borrow().read_slices(&[OBJECT_DATA, object_id])
    }

    pub fn prove_given(
        &self,
        given: Given,
        prop: Proposition,
    ) -> Result<Proof, ProofError<D::Error>> {
        match (given, prop) {
            (Given::RootIs(root), Proposition::ObjectIsValue(object_id, value)) => {
                if root != self.root_hash()? {
                    return Err(ProofError::RootHashMismatch);
                }

                let data = self
                    .object_data(&object_id)?
                    .ok_or(ProofError::MissingObject)?;
                if value != data {
                    return Err(ProofError::ValueMismatch);
                }
                let needed_hash = object_hash(&object_id, &data);

                self.prove_given(
                    Given::RootIs(root),
                    Proposition::HashInObjectTree(needed_hash),
                )
            }
            (Given::RootIs(root), Proposition::HashInObjectTree(hash)) => {
                if root == hash {
                    return Ok(Proof::Empty);
                }

                unimplemented!("hash_in_object_tree");
            }
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum InitError<E> {
    Db(#[from] E),
    IdExists,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ProofError<E> {
    Db(#[from] E),
    RootHashMismatch,
    MissingObject,
    ValueMismatch,
}

pub fn object_hash(object_id: &[u8], object_data: &[u8]) -> Hash {
    use blake2::{Blake2b512, Digest};

    let mut hasher = Blake2b512::default();
    hasher.update(object_id);
    hasher.update(Blake2b512::digest(object_data));
    hasher.finalize().into()
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

pub struct PrefixedHandle<D> {
    prefix: Vec<u8>,
    handle: Handle<D>,
}

impl<D> PrefixedHandle<D> {
    pub fn new(prefix: &[u8], handle: &Handle<D>) -> Self {
        PrefixedHandle {
            prefix: prefix.to_vec(),
            handle: handle.clone(),
        }
    }
}

impl<D> Clone for PrefixedHandle<D> {
    fn clone(&self) -> Self {
        PrefixedHandle {
            prefix: self.prefix.clone(),
            handle: self.handle.clone(),
        }
    }
}

impl<D: Database> Database for PrefixedHandle<D> {
    type Error = D::Error;

    fn store(&mut self, key: &[u8], value: &[u8]) -> Result<(), D::Error> {
        self.handle
            .borrow_mut()
            .store_slices(&[&self.prefix, key], value)
    }

    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, D::Error> {
        self.handle.borrow().read_slices(&[&self.prefix, key])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{kv_set::MockKvSet, test::InMemoryDb};
    use mockall::predicate::*;

    fn mock_kv_set(expectations: impl FnOnce(&mut MockKvSet<InMemoryDb>)) -> MockKvSet<InMemoryDb> {
        let mut kv_set = MockKvSet::default();
        expectations(&mut kv_set);
        kv_set.expect_insert().return_const(Ok(()));
        kv_set.expect_contains().return_const(Ok(false));
        kv_set
    }

    #[cfg(test)]
    mod init {
        use super::*;

        #[test]
        fn stores_bytes_in_value() {
            let db = Handle::new(InMemoryDb::default());
            let mut store = FractalStore::new_with_deps(db.clone(), mock_kv_set(|_| {}));

            store.init(&[42], b"foo").unwrap();

            assert_eq!(db.borrow().read(&[1, 42]).unwrap(), Some(b"foo".to_vec()));
        }

        #[test]
        fn adds_item_id_to_kv_storage_set() {
            let kv_set = mock_kv_set(|kv_set| {
                kv_set
                    .expect_insert()
                    .return_const(Ok(()))
                    .with(eq(&[42][..]))
                    .once();
            });

            let db = Handle::new(InMemoryDb::default());
            let mut store = FractalStore::new_with_deps(db.clone(), kv_set);

            store.init(&[42], b"foo").unwrap();
        }

        #[test]
        fn fails_if_id_exists() {
            let kv_set = mock_kv_set(|kv_set| {
                kv_set
                    .expect_contains()
                    .with(eq(&[42][..]))
                    .return_const(Ok(true));
            });

            let db = Handle::new(InMemoryDb::default());
            let mut store = FractalStore::new_with_deps(db.clone(), kv_set);

            assert_eq!(store.init(&[42], b"foo"), Err(InitError::IdExists));
            assert_eq!(db.borrow().read(&[1, 42]).unwrap(), None);
        }
    }
}
