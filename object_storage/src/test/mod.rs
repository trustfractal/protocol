use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::*;

#[derive(Debug, Default)]
pub struct InMemoryDb {
    stored: HashMap<Vec<u8>, Vec<u8>>,
}

impl InMemoryDb {
    pub fn keys(&self) -> impl Iterator<Item = &Vec<u8>> {
        self.stored.keys()
    }
}

impl Database for InMemoryDb {
    type Error = ();

    fn store(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        self.stored.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self
            .stored
            .get(&key.into_iter().cloned().collect::<Vec<_>>())
            .map(|slice| slice.to_vec()))
    }
}

impl Database for Rc<RefCell<InMemoryDb>> {
    type Error = <InMemoryDb as Database>::Error;

    fn store(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        self.borrow_mut().store(key, value)
    }

    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        self.borrow().read(key)
    }
}
