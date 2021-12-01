#!/usr/bin/env node

const args = require("args-parser")(process.argv);
const { ApiPromise, WsProvider } = require("@polkadot/api");
const types = require(`${process.cwd()}/blockchain/types.json`);
const postgresLib = require('postgres');

async function main() {
  const { chain, postgres: postgresUrl } = args;
  if (!chain || !postgresUrl) {
    console.log("Please provide --chain=wss://yourchain and --postgres=postgres://some-postgres");
    process.exit(1);
  }
  const useSsl = args.ssl === 'true';

  const postgres = await postgresLib(postgresUrl, { ssl: useSsl });
  const storage = await PostgresStorage.create(postgres);

  const provider = new WsProvider(chain);
  const api = await ApiPromise.create({
    provider,
    types,
  });

  updateLargestBlock(api, storage);

  while (true) {
    const didWork = await catchUpToLatest(api, storage);
    if (!didWork) {
      await sleep(6000);
    }
  }
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
    await postgres`
      CREATE TABLE IF NOT EXISTS
      key_values (
        key VARCHAR PRIMARY KEY,
        value VARCHAR NOT NULL
      )
    `;
    return new PostgresStorage(postgres);
  }

  async fullyIngested() {
    const val = await this.getKey('ingestion/fully_ingested');
    return val && Number(val);
  }

  async getKey(key) {
    const [val] = await this.pg`SELECT value FROM key_values WHERE key = ${key} LIMIT 1`;
    return val?.value;
  }

  async setKey(key, value) {
    await this.pg`
      INSERT INTO key_values (key, value) VALUES (${key}, ${value})
      ON CONFLICT (key) DO UPDATE SET value=${value}
    `;
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

async function updateLargestBlock(api, storage) {
  while (true) {
    const latestBlockNumber = (await api.rpc.chain.getBlock()).block.header.number.toNumber();
    await storage.setLargestBlock(latestBlockNumber);
    console.log(`Largest block ${latestBlockNumber}`)
    await sleep(6000);
  }
}

async function sleep(millis) {
  return new Promise(resolve => setTimeout(resolve, millis));
}

async function catchUpToLatest(api, storage) {
  const latestBlockNumber = (await api.rpc.chain.getBlock()).block.header.number.toNumber();
  const fullyIngestedBlock = await storage.fullyIngested();

  if (fullyIngestedBlock != null && fullyIngestedBlock >= latestBlockNumber) {
    return false;
  }

  const maxInProgress = 100;
  const maxToDo = maxInProgress * 100;

  const nextBlock = fullyIngestedBlock == null ? 0 : fullyIngestedBlock + 1;
  const lastBlock = Math.min(latestBlockNumber, nextBlock + maxToDo);
  const blockIngestions = limitedParallel(nextBlock, lastBlock, maxInProgress, async (i) => {
    await ingestBlock(i, api, storage);
  });

  for (const ingestion of blockIngestions) {
    const finished = await ingestion;
    await storage.setFullyIngested(finished);
    if (blockIngestions.length < maxInProgress || finished % maxInProgress === 0) {
      console.log(`Finished ingesting block ${finished}`);
    }
  }

  return true;
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
