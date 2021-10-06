import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';

import { TxnBatcher } from '../lib/TxnBatcher';
import settings from '../settings';
import types from '../../../../../blockchain/types.json'

async function createPromiseApi (nodeAddress: string, types: any) {
  const provider = new WsProvider(nodeAddress);
  const api: ApiPromise = new ApiPromise({ provider, types });

  await api.isReady;

  return api;
}

async function main () {
  const fractalId = 1;
  const api = await createPromiseApi(settings.nodeAddress, types);
  const batcher = new TxnBatcher(api);

  const keyring = new Keyring({ type: 'sr25519' });
  const signer = keyring.createFromUri('//Alice');
  const address = keyring.createFromUri('//Bob').address;

  batcher.send(api.tx.fractalMinting.registerIdentity(fractalId, address), signer);
  setTimeout(() => { console.log('passed'); }, 10000);
}

main().catch(console.error);
