import { ExtensionSetupData } from '@services/ExtensionSetup';

import { Message } from './Message';
export class InjectionScript {
  async getFractalData(substrateAddress: string): Promise<ExtensionSetupData> {
    //TODO: sync all the extensions that contain Fractal Protocol.
    return {
      extensionId: chrome.runtime.id,
      isMain: true,
      substrateAddress,
    };
  }
  async initialize(substrateAddress: string): Promise<void> {
    const script = document.createElement('script');
    script.setAttribute('type', 'text/javascript');
    script.setAttribute(
      'id',
      'fractal-protocol-injected-script-' + chrome.runtime.id
    );
    const data = await this.getFractalData(substrateAddress);
    const src = `
            console.log('Injected script from Fractal Protocol with chrome extension id: ${
              chrome.runtime.id
            }');
            if(!window.hasOwnProperty('GLOBAL_FRACTAL_PROTOCOL_DATA')) {
                GLOBAL_FRACTAL_PROTOCOL_DATA = ${JSON.stringify(data)};
                console.log('Initialized Fractal global variable.')
            }
            setTimeout(() => {
                chrome.runtime.sendMessage('${chrome.runtime.id}', {
                        type: '${Message.INITIALIZE}',
                        extensionData: GLOBAL_FRACTAL_PROTOCOL_DATA,
                    },
                    () => { console.log('sent message from script')}
                );
            }, 1000)

        `;
    script.innerHTML = src;

    document.head.appendChild(script);
  }

  sendCurrentPageView() {
    chrome.runtime.sendMessage({
      type: Message.PAGE_VIEW,
      content: window.location,
    });
  }
}
