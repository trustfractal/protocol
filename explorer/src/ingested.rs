use postgres::Client;

pub fn latest(pg: &mut Client) -> anyhow::Result<Option<u64>> {
    if let Some(v) = get_key("ingestion/fully_ingested", pg)? {
        Ok(Some(v.parse()?))
    } else {
        Ok(None)
    }
}

fn get_key(key: impl AsRef<str>, pg: &mut Client) -> anyhow::Result<Option<String>> {
    let row = match pg
        .query(
            "SELECT value FROM key_values WHERE key = $1",
            &[&key.as_ref()],
        )?
        .into_iter()
        .next()
    {
        Some(row) => row,
        None => return Ok(None),
    };

    Ok(Some(row.get::<_, &str>(&"value").to_string()))
}

pub fn load_block(number: u64, pg: &mut Client) -> anyhow::Result<Option<crate::data::Block>> {
    if let Some(value) = get_key(format!("block/{}", number), pg)? {
        Ok(Some(serde_json::from_str(&value)?))
    } else {
        Ok(None)
    }
}

pub fn load_extrinsic(
    block: u64,
    index: u64,
    pg: &mut Client,
) -> anyhow::Result<Option<crate::data::Extrinsic>> {
    if let Some(value) = get_key(format!("block/{}/extrinsic/{}", block, index), pg)? {
        Ok(Some(serde_json::from_str(&value)?))
    } else {
        Ok(None)
    }
}
