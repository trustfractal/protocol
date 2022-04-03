use fractal_object_storage::{test::InMemoryDb, FractalStore, Given, ProofChecker, Proposition};

#[test]
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
