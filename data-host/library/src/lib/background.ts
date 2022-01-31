import { ExtensionSetupData } from '@services/ExtensionSetup';
import {
  getDataHost,
  getExtensionSetup,
  getMintingRegistrar,
  getProtocolOptIn,
} from '@services/Factory';

import { Message } from './Message';

export class Background {
  async initialize(data: ExtensionSetupData): Promise<void> {
    console.log('Received data: ', data);
    if (!(await getExtensionSetup().isExtensionSetup())) {
      await getExtensionSetup().setupExtensionData(data);
      console.log('Background initialized.');
    }
  }
  async addWebpage(url: string): Promise<void> {
    if (!(await getProtocolOptIn().isOptedIn())) {
      return;
    }
    await getProtocolOptIn().checkOptIn();
    await getDataHost().storeFact({
      pageView: {
        url,
        timestampMs: new Date().getTime(),
      },
    });

    await getMintingRegistrar().maybeTryRegister();
  }
  addListeners() {
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const self = this;
    console.log(chrome.runtime.id);
    chrome.runtime.onMessageExternal.addListener(
      async (request, _sender, sendResponse) => {
        console.log('external message: ', request);
        switch (request.type) {
          case Message.INITIALIZE: {
            await self.initialize(request.extensionData);
            break;
          }
        }

        sendResponse();
      }
    );

    chrome.runtime.onMessage.addListener(
      async (request, _sender, sendResponse) => {
        console.log('internal message: ', request);
        switch (request.type) {
          case Message.PAGE_VIEW: {
            await self.addWebpage(request.content.hostname);
            break;
          }
        }

        sendResponse();
      }
    );
  }
}
