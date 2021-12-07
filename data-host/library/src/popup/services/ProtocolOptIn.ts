import { NotConnectedError } from '@services/FractalAccount';
import { ProtocolService } from '@services/Protocol';
import { Storage } from '@utils/StorageArray';

export class MissingLiveness extends Error {}

export class ProtocolOptIn {
  public postOptInCallbacks: Array<(mnemonic: string) => Promise<void>> = [];

  private completedLivenessOverride = false;

  constructor(
    private readonly storage: Storage,
    private readonly protocol: ProtocolService
  ) {}

  async isOptedIn() {
    return await this.storage.hasItem(await this.mnemonicKey());
  }

  private async mnemonicKey() {
    const network = 'network'; //TODO(melatron): get the current network
    return `opt-in/${network}/mnemonic`;
  }

  async hasCompletedLiveness() {
    if (this.completedLivenessOverride) return true;

    try {
      return await this.protocol.isIdentityRegistered();
    } catch {
      return false;
    }
  }

  async getMnemonic() {
    return await this.storage.getItem(await this.mnemonicKey());
  }

  async optIn(mnemonic: string) {
    await this.storage.setItem(await this.mnemonicKey(), mnemonic);
    await this.runCallbacks(mnemonic);
    await this.tryRegisterIdentity();
  }

  private async runCallbacks(mnemonic: string) {
    while (this.postOptInCallbacks.length > 0) {
      const cb = this.postOptInCallbacks.shift()!;
      await cb(mnemonic);
    }
  }

  async postOptInLiveness() {
    await this.tryRegisterIdentity(async () => {
      //TODO(melatron): implement the windows service
      //   await this.windows.openTab(this.livenessUrl);
    });
  }

  async checkOptIn() {
    const mnemonic = await this.getMnemonic();
    if (mnemonic == null) return;
    await this.runCallbacks(mnemonic);
  }

  private async tryRegisterIdentity(onMissingLiveness?: () => Promise<void>) {
    try {
      await this.protocol.ensureIdentityRegistered();
      this.completedLivenessOverride = true;
    } catch (e) {
      if (e instanceof MissingLiveness) {
        if (onMissingLiveness != null) {
          await onMissingLiveness();
        }
        return;
      }

      if (e instanceof NotConnectedError) {
        return;
      }

      throw e;
    }
  }
}
