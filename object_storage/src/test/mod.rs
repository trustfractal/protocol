use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::*;

#[derive(Default)]
pub struct InMemoryDb {
    stored: HashMap<Vec<u8>, Vec<u8>>,
}

impl Database for InMemoryDb {
    type Error = ();

    fn store<'b>(
        &mut self,
        key: impl IntoIterator<Item = &'b u8>,
        value: &[u8],
    ) -> Result<(), Self::Error> {
        self.stored
            .insert(key.into_iter().cloned().collect::<Vec<_>>(), value.to_vec());
        Ok(())
    }

    fn read<'b>(
        &self,
        key: impl IntoIterator<Item = &'b u8>,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self
            .stored
            .get(&key.into_iter().cloned().collect::<Vec<_>>())
            .map(|slice| slice.to_vec()))
    }
}

impl Database for Rc<RefCell<InMemoryDb>> {
    type Error = <InMemoryDb as Database>::Error;

    fn store<'b>(
        &mut self,
        key: impl IntoIterator<Item = &'b u8>,
        value: &[u8],
    ) -> Result<(), Self::Error> {
        self.borrow_mut().store(key, value)
    }

    fn read<'b>(
        &self,
        key: impl IntoIterator<Item = &'b u8>,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        self.borrow().read(key)
    }
}
