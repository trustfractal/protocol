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
    if (!(await getExtensionSetup().isExtensionSetup())) {
      await getExtensionSetup().setupExtensionData(data);
      console.log('Background initialized.');
    }
  }
  async addWebpage(url: string): Promise<void> {
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
    //TODO(melatron): Create more generic method that handles all type of messages (for different facts)
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const self = this;
    chrome.runtime.onMessage.addListener(
      async (request, _sender, sendResponse) => {
        switch (request.type) {
          case Message.PAGE_VIEW: {
            await self.addWebpage(request.content.hostname);
            break;
          }
          case Message.INITIALIZE: {
            await self.initialize(request.extensionData);
            break;
          }
        }

        sendResponse();
      }
    );
  }
}
