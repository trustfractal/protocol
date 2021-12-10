import { getProtocolService } from '@services/Factory';
import {
  CannotExtendDataset,
  IdentityRegistrationFailed,
  MintingRegistrationFailed,
} from '@services/Protocol';
import { Storage, withLock } from '@utils/StorageArray';

export class MintingRegistrar {
  constructor(
    private readonly storage: Storage,
    private readonly statusCheckSleepSeconds: number
  ) {}

  async maybeTryRegister() {
    const now = new Date().getTime() / 1000;

    return await this.tryRegister(now, async () => {
      const lastCheck = await this.storage.getItem(this.key('last_check'));
      const shouldCheck =
        lastCheck == null ||
        now > parseInt(lastCheck) + this.statusCheckSleepSeconds;
      return shouldCheck;
    });
  }

  async tryRegister(
    maybeNow?: number,
    shouldContinue?: () => Promise<boolean>
  ) {
    if (shouldContinue && !(await shouldContinue())) return;
    const now = maybeNow ?? new Date().getTime() / 1000;

    try {
      await withLock(this.storage, this.key('lock'), async () => {
        // Another "thread" may have registered between our first check and when
        // this acquired the lock.
        if (shouldContinue && !(await shouldContinue())) return;

        try {
          const protocol = await getProtocolService();
          await protocol.ensureIdentityRegistered();
          const isRegistered = await protocol.isRegisteredForMinting();
          if (isRegistered) {
            console.log(
              'Already registered for next minting, not doing anything'
            );
          } else {
            console.log('Not registered for minting, trying to register');
            const hash = await protocol.registerForMinting();
            console.log(`Successfully registered for minting ${hash}`);
          }
          await this.clearLatestError();
        } catch (e) {
          if (e instanceof CannotExtendDataset) {
            this.setLatestFromError(e);
          } else {
            throw e;
          }
        }

        await this.storage.setItem(this.key('last_check'), now.toString());
      });
    } catch (e: any) {
      await this.setLatestFromError(e);
      if (
        e instanceof IdentityRegistrationFailed ||
        e instanceof MintingRegistrationFailed
      ) {
        chrome.runtime.reload();
      } else {
        throw e;
      }
    }
  }

  private key(key: string): string {
    return `minting_registrar/${key}`;
  }

  private async setLatestError(e: MintingError) {
    await this.storage.setItem(this.key('latest_error'), JSON.stringify(e));
  }

  private async setLatestFromError(e: Error) {
    console.error(e);

    let error: MintingError;
    if (e instanceof IdentityRegistrationFailed) {
      error = { type: 'identity_registration' };
    } else if (e instanceof MintingRegistrationFailed) {
      error = { type: 'minting_registration' };
    } else if (e instanceof CannotExtendDataset) {
      error = { type: 'cant_extend_dataset' };
    } else {
      error = { type: 'unknown', message: e.message };
    }
    await this.setLatestError(error);
  }

  private async clearLatestError() {
    await this.storage.removeItem(this.key('latest_error'));
  }

  async latestError(): Promise<MintingError | null> {
    const error = await this.storage.getItem(this.key('latest_error'));
    if (error == null) return null;

    return JSON.parse(error);
  }
}

export type MintingError =
  | { type: 'identity_registration' }
  | { type: 'minting_registration' }
  | { type: 'cant_extend_dataset' }
  | {
      type: 'unknown';
      message: string;
    };
