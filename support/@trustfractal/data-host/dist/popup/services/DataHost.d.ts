import { Storage } from '@utils/StorageArray';
export declare class DataHost {
    private readonly metadata;
    private readonly sensitive;
    private build;
    private extend_multiple;
    private prune_balanced;
    private strict_extension_proof;
    constructor(metadata: Storage, sensitive: Storage);
    private key;
    init(): Promise<void>;
    enable(): Promise<void>;
    disable(): Promise<void>;
    isEnabled(): Promise<boolean>;
    storeFact(fact: any): Promise<void>;
    private array;
    iter(): {
        [Symbol.asyncIterator](): {
            i: number;
            next(): Promise<{
                done: boolean;
                value?: undefined;
            } | {
                value: any;
                done: boolean;
            }>;
        };
    };
    iterBack(): {
        [Symbol.asyncIterator](): {
            i: Promise<number>;
            next(): Promise<{
                done: boolean;
                value?: undefined;
            } | {
                value: any;
                done: boolean;
            }>;
        };
    };
    extensionProof(latestProof: string | null): Promise<string | undefined>;
    currentTree(): Promise<string | undefined>;
    buildTree(items: string[]): string;
}
