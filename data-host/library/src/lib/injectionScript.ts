import { ExtensionSetupData } from '@services/ExtensionSetup';

import { Message } from './Message';
export class InjectionScript {
  getFractalData(substrateAddress: string): ExtensionSetupData {
      //TODO: sync all the extensions that contain Fractal Protocol.
      return {
        extensionId: chrome.runtime.id,
        isMain: true,
        substrateAddress,
      };
  }
  async initialize(substrateAddress: string): Promise<void> {
    const data = this.getFractalData(substrateAddress);
    return new Promise((resolve, reject) => {
      chrome.runtime.sendMessage(
        {
          type: Message.INITIALIZE,
          extensionData: data,
        },
        (response: any) => {
          if (!response) {
            reject();
          }

          console.log(
            `InjectionScript initialized with address: ${substrateAddress}.`
          );
          resolve();
        }
      );
    });
  }

  sendCurrentPageView() {
    chrome.runtime.sendMessage({
      type: Message.PAGE_VIEW,
      content: window.location,
    });
  }
}
