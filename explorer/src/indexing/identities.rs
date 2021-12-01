use super::Indexer;
use crate::data::Extrinsic;

#[derive(Default)]
pub struct CountIdentities {
    current_block: Option<u64>,
}

impl Indexer for CountIdentities {
    fn begin_block(&mut self, number: u64) {
        self.current_block = Some(number);
    }

    fn visit_extrinsic(&mut self, extrinsic: &Extrinsic) {
        let relevant = extrinsic.section == "fractalMinting"
            && extrinsic.method == "registerIdentity"
            && extrinsic.success;
        if !relevant {
            return;
        }

        todo!();
    }
}
