export class InjectionScript {
  // substrateAddress is going to be used to send funds
  async setup(substrateAddress: string): Promise<void> {
    console.log(
      `InjectionScript initialized with address: ${substrateAddress}.`
    );
  }
  sendCurrentPageView() {
    //TODO(melatron): add type as a Enum with all the different facts we would send as a message
    chrome.runtime.sendMessage({ type: 'pageView', content: window.location });
  }
}
