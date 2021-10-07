import {ApiPromise} from '@polkadot/api';

export class TxnBatcher {
  private readonly nextNonces = new Map<string, number|Promise<number>>();

  constructor(private readonly api: ApiPromise) { this.api; }

  signAndSend(txn: any, signer: any, watcher = new TxnWatcher()): TxnWatcher {
    (async () => {
      const nonce = await this.nextNonce(signer);
      console.log('nonce', nonce);
      try {
        const unsub =
            await txn.signAndSend(signer, {nonce}, watcher.signAndSendCb());
        watcher.unsub = unsub;
      } catch (e) {
        if (e instanceof Error && e.message.includes('Priority is too low')) {
          console.log('Conflicting nonce', nonce, 'retrying');
          this.clearNextNonce(signer);
          this.signAndSend(txn, signer, watcher);
        } else {
          throw e;
        }
      }
    })();

    return watcher;
  }

  private async nextNonce(signer: any): Promise<number> {
    while (true) {
      const already = this.nextNonces.get(signer.address);
      if (already instanceof Promise) {
        await already;
      } else if (already == null) {
        const promise = this.api.rpc.system.accountNextIndex(signer.address)
                            .then(n => n.toNumber());
        this.nextNonces.set(signer.address, promise);
        const nonce = await promise;
        this.nextNonces.set(signer.address, nonce + 1);
        return nonce;
      } else {
        this.nextNonces.set(signer.address, already + 1);
        return already;
      }
    }
  }

  private clearNextNonce(signer: any) {
    this.nextNonces.delete(signer.address);
  }
}

export class TxnWatcher {
  unsub?: () => void;

  onInBlock: Array<() => void> = [];

  signAndSendCb(): (result: any) => void {
    return (result: any) => {
      if (result.status.isInBlock) {
        for (const cb of this.onInBlock) {
          cb();
        }
      } else {
        console.log('result.status.toHuman()', result.status.toHuman());
      }
    };
  }

  async inBlock() {
    return new Promise<void>(resolve => { this.onInBlock.push(resolve); });
  }
}
