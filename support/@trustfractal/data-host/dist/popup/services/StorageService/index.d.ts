export declare class StorageService {
    hasItem(key: string): Promise<boolean>;
    getItem(key: string, ifNull?: string): Promise<string | undefined>;
    setItem(key: string, value: string): Promise<void>;
    removeItem(key: string): Promise<void>;
    clear(): Promise<void>;
}
