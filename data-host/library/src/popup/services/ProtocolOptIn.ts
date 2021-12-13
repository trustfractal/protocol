import { IdentityRegistrationFailed, ProtocolService } from '@services/Protocol';
import { WindowsService } from "@services/WindowsService";
import { Storage } from '@utils/StorageArray';

export class MissingLiveness extends Error {}

export class ProtocolOptIn {
  public postOptInCallbacks: Array<(mnemonic: string) => Promise<void>> = [];

  private completedLivenessOverride = false;

  constructor(
    private readonly storage: Storage,
    private readonly protocol: ProtocolService,
    private readonly windows: WindowsService,
    private readonly livenessUrl: string,
  ) {}

  async isOptedIn() {
    return await this.storage.hasItem(this.mnemonicKey());
  }

  private mnemonicKey() {
    //TODO(melatron): old key for mnemonic was opt-in/{maguro.getNetwork()}/mnemonic
    return `opt-in/mnemonic`;
  }

  getAddress(): string {
      return this.protocol.address()
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
    return await this.storage.getItem(this.mnemonicKey());
  }

  async optIn(mnemonic: string) {
    await this.storage.setItem(this.mnemonicKey(), mnemonic);
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
        console.log(this.livenessUrl + this.getAddress());
        console.log(this.windows)
        await this.windows.openTab(this.livenessUrl + this.getAddress());
    });
  }
  //TODO(melatron) the background process for adding facts uses this check.
  async checkOptIn() {
    const mnemonic = await this.getMnemonic();
    if (mnemonic == null) return;
    await this.runCallbacks(mnemonic);
  }

  private async tryRegisterIdentity(onMissingLiveness?: () => Promise<void>) {
    try {
        console.log('--------------------------------------------')
      await this.protocol.ensureIdentityRegistered();
      this.completedLivenessOverride = true;
    } catch (e) {
      if (e instanceof IdentityRegistrationFailed) {
        if (onMissingLiveness != null) {
          await onMissingLiveness();
        }
        return;
      }

      throw e;
    }
  }
}
