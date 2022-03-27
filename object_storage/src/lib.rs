pub type Hash = ();

pub enum Given {
    RootIs(Hash),
}

pub enum Proposition {
    ObjectIsValue(Vec<u8>, Vec<u8>),
}

pub struct Proof {}

impl Proof {
    pub fn serialize(&self) -> Vec<u8> {
        unimplemented!("serialize");
    }
}

pub struct FractalStore<D> {
    db: D,
}

impl<D: Database> FractalStore<D> {
    pub fn new(db: D) -> Self {
        FractalStore { db }
    }

    pub fn init(&mut self, id: &[u8], value: &[u8]) -> Result<(), D::Error> {
        self.db.store(id, value)?;
        Ok(())
    }

    pub fn read(&self, id: &[u8]) -> Result<Option<Vec<u8>>, D::Error> {
        self.db.read(id)
    }

    pub fn root_hash(&self) -> Result<Hash, D::Error> {
        unimplemented!("root_hash");
    }

    pub fn prove_given(&self, given: Given, prop: Proposition) -> Result<Proof, D::Error> {
        unimplemented!("prove_given");
    }
}

pub enum Error<E> {
    DbError(E),
}

pub trait Database {
    type Error;

    fn store(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn keys<'s>(&'s self) -> Box<dyn Iterator<Item = Result<&'s [u8], Self::Error>> + 's>;
}

pub struct ProofChecker {}

impl ProofChecker {
    pub fn new(given: Given) -> Self {
        unimplemented!("new");
    }

    pub fn verify(&self, proposition: Proposition, proof: &[u8]) -> bool {
        unimplemented!("verify");
    }
}
