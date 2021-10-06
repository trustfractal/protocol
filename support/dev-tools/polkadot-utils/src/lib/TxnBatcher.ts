export class TxnBatcher {
  private isInSend = false;
  private queued = new Map();

  constructor (private readonly api: any) {
    this.api = api;
  }

  public async send (txn: any, signer: any) {
    if (!this.queued.has(signer)) {
      this.queued.set(signer, []);
    }

    this.queued.get(signer).push(txn);
    // console.log("Trying to call sendQueued from send");
    await this.sendQueued();
  }

  private async sendQueued () {
    if (this.isInSend) return;
    if (this.queued.size === 0) return;

    // console.log("Called sendQueued");
    this.isInSend = true;
    const signer = Array.from(this.queued.keys())[0];
    const txns = this.queued.get(signer);

    this.queued.delete(signer);

    const result = await this.api.tx.utility
      .batch(txns)
      .signAndSend(signer, ({ status }: { status: any }) => {
        if (status.isInBlock) {
          console.log(`included in ${status.asInBlock}`);
        }
      });

    console.log(result);

    this.isInSend = false;

    this.sendQueued();
  }
}
