#!/usr/bin/env node

const args = require("args-parser")(process.argv);
const { ApiPromise, WsProvider } = require("@polkadot/api");
const types = require(`${process.cwd()}/blockchain/types.json`);
const { Client: PostgresClient } = require('pg');

async function main() {
  const { chain, postgres: postgresUrl } = args;
  if (!chain || !postgresUrl) {
    console.log("Please provide --chain=wss://yourchain and --postgres=postgres://some-postgres");
    process.exit(1);
  }

  const pgClient = new PostgresClient({
    connectionString: postgresUrl,
    ssl: {
      rejectUnauthorized: false
    }
  });
  await pgClient.connect();

  const storage = await PostgresStorage.create(pgClient);

  const provider = new WsProvider(chain);
  const api = await ApiPromise.create({
    provider,
    types,
  });

  let catchingUp = false;
  await onNewBlock(api, async (number) => {
    await storage.setLargestBlock(number);
    console.log(`Largest block ${number}`)

    if (!catchingUp) {
      catchingUp = true;
      await catchUpTo(number, api, storage);
      catchingUp = false;
    }
  });
}

main()
  .then(() => process.exit(0))
  .catch(err => {
    console.error(err);
    process.exit(1);
  });

class PostgresStorage {
  constructor(postgres) {
    this.pg = postgres;
  }

  static async create(postgres) {
    await postgres.query(`
      CREATE TABLE IF NOT EXISTS
      key_values (
        key VARCHAR PRIMARY KEY,
        value VARCHAR NOT NULL
      )
    `);
    return new PostgresStorage(postgres);
  }

  async fullyIngested() {
    const val = await this.getKey('ingestion/fully_ingested');
    return val && Number(val);
  }

  async getKey(key) {
    const val = await this.pg.query(`SELECT value FROM key_values WHERE key = '${key}' LIMIT 1`);
    return val.rows[0]?.value;
  }

  async setKey(key, value) {
    await this.pg.query(`
      INSERT INTO key_values (key, value) VALUES ('${key}', '${value}')
      ON CONFLICT (key) DO UPDATE SET value='${value}'
    `);
  }

  async setFullyIngested(number) {
    await this.setKey('ingestion/fully_ingested', number.toString());
  }

  async setLargestBlock(number) {
    await this.setKey('ingestion/largest', number.toString());
  }

  async saveBlock(number, blockData) {
    const json = JSON.stringify(blockData);
    await this.setKey(`block/${number}`, json);
  }

  async saveExtrinsic(blockNumber, index, extrinsicData) {
    const json = JSON.stringify(extrinsicData);
    await this.setKey(`block/${blockNumber}/extrinsic/${index}`, json);
  }
}

async function onNewBlock(api, callback) {
  await api.rpc.chain.subscribeNewHeads(async (header) => {
    const latestBlockNumber = header.number.toNumber();
    callback(latestBlockNumber);
  });
  return new Promise(resolve => {});
}

async function sleep(millis) {
  return new Promise(resolve => setTimeout(resolve, millis));
}

async function catchUpTo(catchUpBlock, api, storage) {
  const fullyIngestedBlock = await storage.fullyIngested();

  if (fullyIngestedBlock != null && fullyIngestedBlock >= catchUpBlock) {
    return;
  }

  const logEvery = new Interval(1000);
  const maxInProgress = 100;
  const maxToDo = maxInProgress * 100;

  const nextBlock = fullyIngestedBlock == null ? 0 : fullyIngestedBlock + 1;
  const lastBlock = Math.min(catchUpBlock, nextBlock + maxToDo);

  const blockIngestions = limitedParallel(nextBlock, lastBlock, maxInProgress, async (i) => {
    await ingestBlock(i, api, storage);
  });

  for (const ingestion of blockIngestions) {
    const finished = await ingestion;
    await storage.setFullyIngested(finished);
    if (logEvery.isTime()) {
      console.log(`Finished ingesting block ${finished}`);
    }
  }
}

class Interval {
  constructor(every) {
    this.every = every;
    this.last = null;
  }

  isTime() {
    const now = new Date();
    if (this.last != null && (now - this.last) < this.every) {
      return false;
    }

    this.last = now;
    return true;
  }
}

// Returns an array of promises that resolve to the values in [start..=end].
// Only `maxInProgress` promises will be running at any moment.
function limitedParallel(start, end, maxInProgress, callback) {
  const resolves = {};
  const rejects = {};

  const result = [];
  for (let i = start; i <= end; i++) {
    result.push(new Promise((resolve, reject) => {
      resolves[i] = () => {
        resolve(i);
      };
      rejects[i] = reject;
    }));
  }

  let inProgress = 0;
  let nextToRun = start;
  (async function() {
    const startNext = async () => {
      if (inProgress >= maxInProgress) return;
      if (nextToRun > end) return;

      inProgress += 1;
      const thisToRun = nextToRun++;
      if (inProgress < maxInProgress) startNext();

      try {
        await callback(thisToRun);
        resolves[thisToRun]();
      } catch (e) {
        rejects[thisToRun](e);
      }

      inProgress -= 1;
      startNext();
    }

    startNext();
  })();

  return result;
}

async function ingestBlock(blockNumber, api, storage) {
  const hash = await api.rpc.chain.getBlockHash(blockNumber);
  const block = (await api.rpc.chain.getBlock(hash)).block;

  const blockData = {
    hash: hash.toHex(),
    parent: block.header.parentHash.toHex(),
    number: block.header.number.toNumber(),
  };
  await storage.saveBlock(blockNumber, blockData);

  const events = await api.query.system.events.at(hash);
  for (let [index, extr] of block.extrinsics.entries()) {
    const extrinsicEvent = events.find(
      (e) => e.phase.isApplyExtrinsic && e.phase.asApplyExtrinsic.eq(index)
    );
    const isFailure = api.events.system.ExtrinsicFailed.is(
      extrinsicEvent.event
    );

    let error;
    if (isFailure) {
      error = extrinsicEvent.toHuman().event.data[0].Module;
    }

    const extrData = {
      block: hash.toHex(),
      index_in_block: index,
      signer: extr.toHuman().signer?.Id,
      nonce: extr.nonce?.toNumber(),
      section: extr.method.section,
      method: extr.method.method,
      args: JSON.parse(JSON.stringify(extr.method.args)),
      success: !isFailure,
      error,
    };

    await storage.saveExtrinsic(blockNumber, index, extrData);
  }
}
