import { Storage } from '@utils/StorageArray';
import { Observable } from 'rxjs';
export declare function useLoadedState<T>(loader: () => Promise<T>, watchedVars?: unknown[]): Load<T>;
export declare type Load<T, Orig = T> = Loading<T, Orig> | Loaded<T, Orig>;
export declare class Loading<T, Orig = T> {
    readonly setValue: (t: Orig) => void;
    readonly reload: () => void;
    isLoaded: false;
    constructor(setValue: (t: Orig) => void, reload: () => void);
    unwrapOrDefault<U>(def: U): U;
    map<U>(_fn: (t: T) => U): Loading<U, Orig>;
}
export declare class Loaded<T, Orig = T> {
    readonly value: T;
    readonly setValue: (t: Orig) => void;
    readonly reload: () => void;
    isLoaded: true;
    constructor(value: T, setValue: (t: Orig) => void, reload: () => void);
    unwrapOrDefault<U>(_def: U): T;
    map<U>(fn: (t: T) => U): Loaded<U, Orig>;
}
export declare function useObservedState<T>(observable: () => Observable<T>): Observed<T>;
export interface Observed<T> {
    hasValue: boolean;
    value?: T;
    unwrapOrDefault<U>(def: U): T | U;
}
export interface CacheArgs<T> {
    cache: ValueCache;
    key: string;
    useFor?: number;
    loader: () => Promise<T>;
    cacheWhen?: (t: T) => boolean;
    onValue?: (t: T) => void;
    serialize?: (t: T) => string;
    deserialize?: (s: string) => T;
}
export declare function useCachedState<T>(args: CacheArgs<T>): Load<T>;
export declare class ValueCache {
    private readonly storage;
    private memory;
    constructor(storage: Storage);
    get(key: string): Promise<[number, string] | null>;
    getImmediate(key: string): [number, string] | null;
    set(key: string, value: string): Promise<void>;
    remove(key: string): Promise<void>;
}
