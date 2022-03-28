use crate::*;

pub(crate) struct KvSet {}

#[cfg_attr(test, mockall::automock)]
impl KvSet {
    pub fn new() -> Self {
        KvSet {}
    }

    pub fn insert<D: Database + 'static>(&self, _key: &[u8], _db: &mut D) -> Result<(), D::Error> {
        unimplemented!("insert");
    }

    pub fn contains<D: Database + 'static>(&self, _key: &[u8], _db: &D) -> Result<bool, D::Error> {
        unimplemented!("contains");
    }
}
