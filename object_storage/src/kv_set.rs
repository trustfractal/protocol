use crate::*;

pub(crate) struct KvSet {
    prefix: Vec<u8>,
}

#[cfg_attr(test, mockall::automock)]
impl KvSet {
    pub fn new(prefix: &[u8]) -> Self {
        KvSet {
            prefix: prefix.to_vec(),
        }
    }

    pub fn insert<D: Database + 'static>(&self, key: &[u8], db: &mut D) -> Result<(), D::Error> {
        db.store(self.prefix.iter().chain(key), &[])
    }

    pub fn contains<D: Database + 'static>(&self, key: &[u8], db: &D) -> Result<bool, D::Error> {
        Ok(db.read(self.prefix.iter().chain(key))?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test::*;

    #[test]
    fn new_contains_nothing() {
        let db = InMemoryDb::default();
        let kv_set = KvSet::new(&[1, 2, 3]);

        assert_eq!(kv_set.contains(b"foo", &db), Ok(false));
    }

    #[test]
    fn insert_contains_key() {
        let mut db = InMemoryDb::default();
        let kv_set = KvSet::new(&[1, 2, 3]);

        kv_set.insert(b"foo", &mut db).unwrap();

        assert_eq!(kv_set.contains(b"foo", &db), Ok(true));
    }

    #[test]
    fn prefixes_db_entries_with_provided_prefix() {
        let mut db = InMemoryDb::default();
        let kv_set = KvSet::new(&[1, 2, 3]);

        kv_set.insert(b"foo", &mut db).unwrap();

        assert!(db.read(&[1, 2, 3, 102, 111, 111]).unwrap().is_some());
    }
}
