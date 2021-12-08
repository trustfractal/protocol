export abstract class MultiContext {
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  inInjectedScript(): void {}
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  inBackground(): void {}
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  inPopup(): void {}
}

export class AggregateMultiContext extends MultiContext {
  constructor(private readonly delegates: Array<MultiContext>) {
    super();
  }

  inInjectedScript(): void {
    this.delegates.forEach((d) => d.inInjectedScript());
  }

  inBackground(): void {
    this.delegates.forEach((d) => d.inBackground());
  }
  inPopup(): void {
    this.delegates.forEach((d) => d.inPopup());
  }
}
