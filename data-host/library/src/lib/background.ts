import {
    getDataHost,
    getMintingRegistrar,
    getProtocolOptIn,
  } from "@services/Factory";


export class Background {
  async setup(): Promise<void> {
    console.log('Background initialized.');
    await getDataHost().init();
    //TODO(melatron): Initialize everything needed for facts storage.
  }
  async addWebpage(url: string): Promise<void> {
    await this.setup();
    await getProtocolOptIn().checkOptIn();
    await getDataHost().storeFact({
      pageView: {
        url,
        timestampMs: new Date().getTime(),
      },
    });

    await getMintingRegistrar().maybeTryRegister();
  }
}
