import { SubmittableExtrinsic } from '@polkadot/api/types';
import { KeyringPair } from '@polkadot/keyring/types';
import { GenericEventData } from '@polkadot/types/generic';
import { DispatchError } from '@polkadot/types/interfaces';
import { AnyJson, ISubmittableResult } from '@polkadot/types/types';

export type TxnError = Error | DispatchError | GenericEventData | AnyJson;

export interface TxnReady {
  // This may change if the TXN needs to be retried.
  hash: string;
}

export interface TxnInBlock {
  block: string;
  hash: string;
}

export interface TxnFinalized {
  includedInBlock: string;
  hash: string;
}

export class TxnWatcher {
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  unsub: () => void = () => {};

  public status: AnyJson | string = 'Unsubmitted';
  private hash?: string;

  private onReady = new MultiCallback<TxnReady>();
  private onInBlock = new OnceMultiCallback<TxnInBlock>('onInBlock');
  private onFinalized = new OnceMultiCallback<TxnFinalized>('onFinalized');

  private onUnhandledError = new OnceMultiCallback<TxnError>(
    'onUnhandledError'
  );

  // eslint-disable-next-line @typescript-eslint/no-empty-function
  handleInvalid: () => void = () => {};

  signAndSendCb(): (result: ISubmittableResult) => void {
    return (result: ISubmittableResult) => {
      if (result.dispatchError) {
        this.status = 'Error';
        this.onError(result.dispatchError.toHuman());
        return;
      }

      if (this.extrinsicEventsError(result)) {
        return;
      }

      if (result.status.isReady) {
        this.status = 'Ready';
        this.onReady.callAll({ hash: this.hash! });
      } else if (result.status.isBroadcast) {
        // Nothing to do when breadcasted, but not handling will trigger the
        // unhandled case below.
      } else if (result.status.isInBlock) {
        this.status = 'InBlock';
        this.onInBlock.callAll({
          block: result.status.asInBlock.toHex(),
          hash: this.hash!,
        });
      } else if (result.status.isFinalized) {
        this.onInBlock.callIfUncalled({
          block: result.status.asFinalized.toHex(),
          hash: this.hash!,
        });

        this.status = 'Finalized';
        this.onFinalized.callAll({
          includedInBlock: result.status.asFinalized.toHex(),
          hash: this.hash!,
        });
        this.unsub();
      } else if (result.status.isFuture) {
        this.status = 'Future';
        // Future means we submitted a TXN with too high of a nonce. Since
        // we probably submitted the previous nonce about the same time, this
        // doesn't end up being a problem.
      } else if (result.status.isInvalid && this.handleInvalid != null) {
        console.log('Handling invalid with callback');
        this.handleInvalid();
      } else {
        this.status = result.status.toHuman();
        const error = new Error(`Unhandled status: ${result.status.toHuman()}`);
        this.onError(error);
      }
    };
  }

  private onError(error: TxnError) {
    console.error(error);
    this.onUnhandledError.callAll(error);
    this.unsub();
  }

  private extrinsicEventsError(result: ISubmittableResult): boolean {
    const events = result.events;
    if (events.length === 0) return false;

    for (const { event } of events) {
      if (event.section !== 'system') continue;

      if (event.method === 'ExtrinsicSuccess') {
        return false;
      }
      if (event.method === 'ExtrinsicFailed') {
        this.status = 'ExtrinsicFailed';
        this.onError(event.data);
        return true;
      }
    }

    const error = new Error(
      `No extrinsic event found for extrinsic: ${
        this.hash
      } in state ${result.status.toHuman()}`
    );
    this.onError(error);
    throw error;
  }

  async ready(): Promise<TxnReady> {
    return this.promise((resolve) => {
      this.onReady.push(resolve);
    });
  }

  private promise<T>(
    withResolve: (resolve: (t: T) => void) => void
  ): Promise<T> {
    return new Promise((resolve, reject) => {
      withResolve(resolve);
      this.onUnhandledError.push(reject);
    });
  }

  async inBlock(): Promise<TxnInBlock> {
    return this.promise((resolve) => {
      this.onInBlock.push(resolve);
    });
  }

  async finalized(): Promise<TxnFinalized> {
    return this.promise((resolve) => {
      this.onFinalized.push(resolve);
    });
  }

  async signAndSend(
    txn: SubmittableExtrinsic<'promise'>,
    signer: KeyringPair,
    options?: { nonce?: number }
  ) {
    const unsub = await txn.signAndSend(signer, options || {}, this.signAndSendCb());
    this.hash = txn.hash.toHex();
    this.unsub = unsub;
  }

  static signAndSend(
    txn: SubmittableExtrinsic<'promise'>,
    signer: KeyringPair,
    options?: { nonce?: number }
  ): TxnWatcher {
    const watcher = new TxnWatcher();
    (async () => {
      try {
        await watcher.signAndSend(txn, signer, options);
      } catch (e) {
        watcher.onError(e);
      }
    })();
    return watcher;
  }
}

class MultiCallback<T> {
  private willBeCalled: Array<(t: T) => void> = [];

  push(callback: (t: T) => void) {
    this.willBeCalled.push(callback);
  }

  callAll(t: T) {
    for (const cb of this.willBeCalled) {
      cb(t);
    }
  }
}

// Container for multiple callbacks that will only be called once.
// Will call new callbacks with the value if added after callAll.
class OnceMultiCallback<T> {
  private value?: T;
  private willBeCalled: Array<(t: T) => void> | null = [];

  constructor(private readonly name: string) {}

  push(callback: (t: T) => void) {
    if (this.willBeCalled == null) {
      callback(this.value!);
    } else {
      this.willBeCalled.push(callback);
    }
  }

  callAll(t: T) {
    this.value = t;

    if (this.willBeCalled == null) {
      throw new Error(`Called callAll(${this.name}) more than once`);
    } else {
      const toCall = this.willBeCalled;
      this.willBeCalled = null;
      for (const cb of toCall) {
        cb(t);
      }
    }
  }

  hasBeenCalled() {
    return this.willBeCalled == null;
  }

  callIfUncalled(t: T) {
    if (this.hasBeenCalled()) return;
    console.warn('Calling uncalled callbacks');
    this.callAll(t);
  }
}
