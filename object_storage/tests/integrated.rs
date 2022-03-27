use std::collections::HashMap;

use fractal_object_storage::{Database, FractalStore, Given, ProofChecker, Proposition};

#[derive(Default)]
struct InMemoryDb {
    stored: HashMap<Vec<u8>, Vec<u8>>,
}

impl Database for InMemoryDb {
    type Error = ();

    fn store(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        self.stored.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.stored.get(key).map(|slice| slice.to_vec()))
    }

    fn keys<'s>(&'s self) -> Box<dyn Iterator<Item = Result<&'s [u8], Self::Error>> + 's> {
        Box::new(self.stored.keys().map(Vec::as_slice).map(Ok))
    }
}

#[test]
#[ignore]
fn proving_simple_value() {
    let storage = InMemoryDb::default();
    let mut fractal_store = FractalStore::new(storage);

    fractal_store.init(&[42], "foo".as_bytes()).unwrap();
    let root = fractal_store.root_hash().unwrap();
    let proof = fractal_store
        .prove_given(
            Given::RootIs(root),
            Proposition::ObjectIsValue(vec![42], b"foo".to_vec()),
        )
        .unwrap();

    assert!(ProofChecker::new(Given::RootIs(root)).verify(
        Proposition::ObjectIsValue(vec![42], b"foo".to_vec()),
        &proof.serialize()
    ));
}
