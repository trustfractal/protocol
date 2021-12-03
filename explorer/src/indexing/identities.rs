use super::*;
use crate::data::Extrinsic;

#[derive(Default)]
pub struct CountIdentities {
    current_block: Option<u64>,
    previous_count: i32,
    this_count: i32,
}

impl Indexer for CountIdentities {
    fn version(&mut self) -> u32 {
        2
    }

    fn version_upgrade(&mut self, pg: &mut Client) -> anyhow::Result<()> {
        pg.execute("DROP TABLE IF EXISTS identity_first_seen", &[])?;
        pg.execute("DROP TABLE IF EXISTS unique_identity_counts", &[])?;

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
        pg.execute(
            "
            CREATE TABLE IF NOT EXISTS
            unique_identity_counts (
                block_number INT PRIMARY KEY,
                id_count INT NOT NULL
            )
        ",
            &[],
        )?;

        Ok(())
    }

    fn begin_block(&mut self, block: &Block, pg: &mut Client) -> anyhow::Result<()> {
        self.current_block = Some(block.number);
        self.previous_count = pg
            .query_opt(
                "SELECT id_count FROM unique_identity_counts WHERE block_number = $1",
                &[&(block.number as i32 - 1)],
            )?
            .map(|row| row.get(&"id_count"))
            .unwrap_or(0);
        self.this_count = 0;

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

        let rows_modified = pg.execute(
            "INSERT INTO identity_first_seen (id, block) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING",
            &[&id, &block],
        )?;
        if rows_modified > 0 {
            self.this_count += 1;
        }

        Ok(())
    }

    fn end_block(&mut self, block: &Block, pg: &mut Client) -> anyhow::Result<()> {
        pg.execute(
            "INSERT INTO unique_identity_counts (block_number, id_count)
            VALUES ($1, $2)
            ON CONFLICT (block_number) DO NOTHING",
            &[
                &(block.number as i32),
                &(self.previous_count + self.this_count),
            ],
        )?;

        Ok(())
    }
}
