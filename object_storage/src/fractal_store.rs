use crate::*;

#[mockall_double::double]
use crate::kv_set::KvSet;

pub struct FractalStore<D> {
    db: D,
    kv_set: KvSet,
}

impl<D: Database + 'static> FractalStore<D> {
    pub fn new(db: D) -> Self {
        Self::new_with_deps(db, KvSet::new())
    }

    fn new_with_deps(db: D, kv_set: KvSet) -> Self {
        FractalStore { db, kv_set }
    }

    pub fn init(&mut self, id: &[u8], value: &[u8]) -> Result<(), Error<D::Error>> {
        if self.kv_set.contains(id, &self.db)? {
            return Err(Error::IdExists);
        }

        self.db.store(
            [1].iter().chain(id).cloned().collect::<Vec<_>>().as_ref(),
            value,
        )?;
        self.kv_set.insert(id, &mut self.db)?;

        Ok(())
    }

    pub fn root_hash(&self) -> Result<Hash, D::Error> {
        unimplemented!("root_hash");
    }

    pub fn prove_given(&self, _given: Given, _prop: Proposition) -> Result<Proof, D::Error> {
        unimplemented!("prove_given");
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum Error<E> {
    DbError(#[from] E),
    IdExists,
}

pub trait Database {
    type Error;

    fn store(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::kv_set::MockKvSet;

    #[cfg(test)]
    mod init {
        use super::*;

        use mockall::predicate::*;
        use std::{cell::RefCell, rc::Rc};

        fn mock_kv_set(expectations: impl FnOnce(&mut MockKvSet)) -> MockKvSet {
            let mut kv_set = MockKvSet::default();
            expectations(&mut kv_set);
            kv_set.expect_insert::<Rc<_>>().return_const(Ok(()));
            kv_set.expect_contains::<Rc<_>>().return_const(Ok(false));
            kv_set
        }

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
