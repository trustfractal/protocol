#!/usr/bin/env node


async function promiseProgress(progress, promises) {
  progress.start(promises.length, 0);
  return Promise.all(
    promises.map(async (p) => {
      const result = await p;
      progress.increment();
      return result;
    })
  );
}

const args = require("args-parser")(process.argv);
const { ApiPromise, WsProvider } = require("@polkadot/api");
const types = require(`${process.cwd()}/blockchain/types.json`);
const redisLib = require('redis');

async function main() {
  const { chain, redis: redisUrl } = args;
  if (!chain || !redisUrl) {
    console.log("Please provide --chain=wss://yourchain and --redis=redis://some-redis");
    process.exit(1);
  }

  const redis = redisLib.createClient({
    url: redisUrl,
  });
  await redis.connect();
  const storage = new RedisStorage(redis);

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

class RedisStorage {
  constructor(redis) {
    this.redis = redis;
  }

  async fullyIngested() {
    const val = await this.redis.get('ingestion/fully_ingested');
    return val && Number(val);
  }

  async setFullyIngested(number) {
    await this.redis.set('ingestion/fully_ingested', number.toString());
  }

  async setLargestBlock(number) {
    await this.redis.set('ingestion/largest', number.toString());
  }

  async saveBlock(number, blockData) {
    const json = JSON.stringify(blockData);
    await this.redis.set(`block/${number}`, json);
  }

  async saveExtrinsic(blockNumber, index, extrinsicData) {
    const json = JSON.stringify(extrinsicData);
    await this.redis.set(`block/${blockNumber}/extrinsic/${index}`, json);
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

  const nextBlock = fullyIngestedBlock == null ? 0 : fullyIngestedBlock + 1;
  const blockIngestions = limitedParallel(nextBlock, latestBlockNumber, maxInProgress, async (i) => {
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
