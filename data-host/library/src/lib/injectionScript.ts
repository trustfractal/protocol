export class InjectionScript {
  // substrateAddress is going to be used to send funds
  async setup(substrateAddress: string): Promise<void> {
    console.log(
      `InjectionScript initialized with address: ${substrateAddress}.`
    );
  }
}
