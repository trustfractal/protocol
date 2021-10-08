import {DispatchError} from '@polkadot/types/interfaces';
import {AnyJson, ISubmittableResult} from '@polkadot/types/types';

export type TxnError = Error|DispatchError;

export class TxnWatcher {
  unsub?: () => void;

  status: AnyJson|string = "Unsubmitted";

  onReady = new MultiCallback<void>();
  onInBlock = new OnceMultiCallback<void>('onInBlock');
  onFinalized = new OnceMultiCallback<void>('onFinalized');

  onUnhandledError = new OnceMultiCallback<TxnError>('onUnhandledError');

  handleInvalid?: () => void;

  signAndSendCb(): (result: ISubmittableResult) => void {
    return (result: ISubmittableResult) => {
      if (result.dispatchError) {
        this.status = "Error";
        this.onError(result.dispatchError);
        return;
      }

      if (result.status.isReady) {
        this.status = "Ready";
        this.onReady.callAll();
      } else if (result.status.isInBlock) {
        this.status = "InBlock";
        this.onInBlock.callAll();
      } else if (result.status.isFinalized) {
        this.onInBlock.callIfUncalled();

        this.status = "Finalized";
        this.onFinalized.callAll();
        this.unsub!();
      } else if (result.status.isFuture) {
        this.status = "Future";
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
    this.unsub!();
  }

  async ready(): Promise<void> {
    return this.promise(resolve => { this.onReady.push(resolve); });
  }

  private promise<T>(withResolve: (resolve: (t: T) => void) => void):
      Promise<T> {
    return new Promise((resolve, reject) => {
      withResolve(resolve);
      this.onUnhandledError.push(reject);
    });
  }

  async inBlock(): Promise<void> {
    return this.promise(resolve => { this.onInBlock.push(resolve); });
  }

  async finalized(): Promise<void> {
    return this.promise(resolve => { this.onFinalized.push(resolve); });
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
  private willBeCalled: Array<(t: T) => void>|null = [];

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

  hasBeenCalled() { return this.willBeCalled == null; }

  callIfUncalled(t: T) {
    if (this.hasBeenCalled())
      return;
    console.warn('Calling uncalled callbacks');
    this.callAll(t);
  }
}