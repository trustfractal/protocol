export interface Storage {
    setItem(key: string, value: string): Promise<void>;
    getItem(key: string): Promise<string | undefined>;
    hasItem(key: string): Promise<boolean>;
    removeItem(key: string): Promise<void>;
}
declare class StringifyStorage {
    private readonly base;
    constructor(base: Storage);
    setItem(key: string, value: any): Promise<void>;
    getItem(key: string): Promise<any>;
    hasItem(key: string): Promise<boolean>;
    removeItem(key: string): Promise<void>;
}
export declare class StorageArray {
    base: StringifyStorage;
    constructor(rootStorage: Storage, prefix: string);
    push(item: any): Promise<number>;
    length(): Promise<number>;
    get(index: number): Promise<any>;
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
}
export declare function withLock<T>(storage: Storage, key: string, callback: () => Promise<T>): Promise<T>;
export {};
