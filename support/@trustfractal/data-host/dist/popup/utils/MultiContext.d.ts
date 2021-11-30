export declare abstract class MultiContext {
    inInjectedScript(): void;
    inBackground(): void;
    inPopup(): void;
}
export declare class AggregateMultiContext extends MultiContext {
    private readonly delegates;
    constructor(delegates: Array<MultiContext>);
    inInjectedScript(): void;
    inBackground(): void;
    inPopup(): void;
}
