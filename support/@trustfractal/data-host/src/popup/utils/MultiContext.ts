export abstract class MultiContext {
  inInjectedScript(): void {}
  inBackground(): void {}
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
