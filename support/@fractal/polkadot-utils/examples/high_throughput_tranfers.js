const {ApiPromise, Keyring, WsProvider} = require('@polkadot/api');
const {TxnBatcher} = require('../build/main');

async function main() {
  const provider = new WsProvider('ws://localhost:9944');
  const api = await ApiPromise.create({ provider });
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  const bob = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';

  const batcher = new TxnBatcher(api);

  const promises = [];
  for (let i = 0; i < 3000; i++) {
    promises.push((async () => {
      const txn = api.tx.balances.transfer(bob, 12345);
      // console.log(`Sending transfer ${i}`);
      await batcher.signAndSend(txn, alice).inBlock();
      // console.log(`Transfer ${i} in block`);
    })());

    await new Promise(resolve => setTimeout(resolve, 10));
  }

  await Promise.all(promises);
}

main()
  .then(() => process.exit(0))
  .catch(e => {
    console.error(e);
    process.exit(1);
  });
