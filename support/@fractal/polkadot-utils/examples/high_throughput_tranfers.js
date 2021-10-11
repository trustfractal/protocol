const {ApiPromise, Keyring, WsProvider} = require('@polkadot/api');
const {TxnBatcher} = require('../build/main');

async function isolatedBatcher() {
  const provider = new WsProvider('ws://localhost:9944');
  const api = await ApiPromise.create({provider});
  return new TxnBatcher(api);
}

async function main() {
  const provider = new WsProvider('ws://localhost:9944');
  const api = await ApiPromise.create({provider});
  const keyring = new Keyring({type : 'sr25519'});
  const alice = keyring.addFromUri('//Alice');

  const bob = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';

  const watchers = new Set();

  let remaining = 3000;
  setInterval(() => {
    const statuses = {};
    for (const txn of watchers) {
      statuses[txn.status] = (statuses[txn.status] || 0) + 1;
    }
    if (remaining > 0) {
      statuses['Unstarted'] = remaining;
    }
    const now = new Date().toLocaleTimeString();
    console.log(now, 'statuses', statuses);
  }, 1000);

  const batchers = [
    await isolatedBatcher(),
    // await isolatedBatcher(),
    // await isolatedBatcher(),
    // await isolatedBatcher(),
  ];

  const promises = [];
  await new Promise(resolve => {
    setInterval(() => {
      if (remaining <= 0) return resolve();
      remaining -= 1;

      promises.push((async () => {
        const txn = api.tx.balances.transfer(bob, 12345);

        const watcher =
            batchers[remaining % batchers.length].signAndSend(txn, alice);
        watchers.add(watcher);
        await watcher.inBlock();
      })());
    }, 10)
  })

  await Promise.all(promises);
  console.log('All txns in block');
  await Promise.all(Array.from(watchers).map(w => w.finalized()));
  console.log('All txns finalized');
}

main().then(() => process.exit(0)).catch(e => {
  console.error(e);
  process.exit(1);
});
