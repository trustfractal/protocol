import { ApiPromise } from '@polkadot/api';
import { SubmittableExtrinsic } from '@polkadot/api/types';
import { KeyringPair } from '@polkadot/keyring/types';

import { TxnWatcher } from './watcher';

export class TxnBatcher {
  private readonly nextNonces = new Map<string, number | Promise<number>>();

  constructor(private readonly api: ApiPromise) {
    this.api;
  }

  signAndSend(
    txn: SubmittableExtrinsic<'promise'>,
    signer: KeyringPair,
    watcher = new TxnWatcher()
  ): TxnWatcher {
    (async () => {
      const retry = () => {
        console.warn('Retrying nonce', nonce);
        this.clearNextNonce(signer);
        this.signAndSend(txn, signer, watcher);
      };

      const nonce = await this.nextNonce(signer);
      try {
        await watcher.signAndSend(txn, signer, { nonce });
        watcher.handleInvalid = retry;
        await watcher.ready();
      } catch (e) {
        if (!(e instanceof Error)) {
          throw e;
        }

        const retryable = [
          'Priority is too low',
          'Transaction is outdated',
        ].some((m) => (e as Error).message.includes(m));
        if (retryable) {
          retry();
        } else {
          console.log('Throwing unhandled error', e);
          throw e;
        }
      }
    })();

    return watcher;
  }

  private async nextNonce(signer: KeyringPair): Promise<number> {
    const address = signer.address;
    /*eslint no-constant-condition: ["error", { "checkLoops": false }]*/
    while (true) {
      const already = this.nextNonces.get(address);
      if (already instanceof Promise) {
        await already;
      } else if (already == null) {
        const promise = this.api.rpc.system
          .accountNextIndex(address)
          .then((n) => n.toNumber());
        this.nextNonces.set(address, promise);
        const nonce = await promise;
        this.nextNonces.set(address, nonce + 1);
        return nonce;
      } else {
        this.nextNonces.set(address, already + 1);
        return already;
      }
    }
  }

  private clearNextNonce(signer: KeyringPair) {
    const already = this.nextNonces.get(signer.address);
    if (typeof already === 'number') {
      this.nextNonces.delete(signer.address);
    }
  }
}
