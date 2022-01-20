import { WindowsService } from '@services/WindowsService';
import { Storage } from '@utils/StorageArray';

export type ExtensionSetupData = {
  extensionId: string;
  isMain: boolean;
  substrateAddress: string;
};

export class ExtensionSetup {
  constructor(
    private readonly storage: Storage,
    private readonly windows: WindowsService,
    private readonly extensionSetupUrl: string
  ) {}

  async isExtensionSetup() {
    return await this.storage.hasItem(this.setupKey());
  }

  private setupKey() {
    return `extension-setup/data`;
  }

  async getExtensionSetup() {
    const data = await this.storage.getItem(this.setupKey());

    return data ? JSON.parse(data) : '';
  }

  async setupExtension() {
    await this.windows.openTab(this.extensionSetupUrl);
  }
  async setupExtensionData(data: ExtensionSetupData) {
    console.log(`Setup with extension data:`, data);
    await this.storage.setItem(this.setupKey(), JSON.stringify(data));
  }
}
