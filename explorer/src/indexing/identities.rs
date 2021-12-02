use super::*;
use crate::data::Extrinsic;

#[derive(Default)]
pub struct CountIdentities {
    current_block: Option<u64>,
}

impl Indexer for CountIdentities {
    fn begin(&mut self, pg: &mut Client) -> anyhow::Result<()> {
        pg.execute(
            "
            CREATE TABLE IF NOT EXISTS
            identity_first_seen (
                id VARCHAR PRIMARY KEY,
                block INT NOT NULL
            )
        ",
            &[],
        )?;

        Ok(())
    }

    fn begin_block(&mut self, block: &Block, _pg: &mut Client) -> anyhow::Result<()> {
        self.current_block = Some(block.number);
        Ok(())
    }

    fn visit_extrinsic(&mut self, extrinsic: &Extrinsic, pg: &mut Client) -> anyhow::Result<()> {
        let relevant = extrinsic.section == "fractalMinting"
            && extrinsic.method == "registerIdentity"
            && extrinsic.success;
        if !relevant {
            return Ok(());
        }

        let id = extrinsic.args[0].as_str().expect("arg 0 is string");
        let block = self.current_block.unwrap() as i32;
        pg.execute(
            "INSERT INTO identity_first_seen (id, block) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING",
            &[&id, &block],
        )?;

        Ok(())
    }
}
