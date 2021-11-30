import { ProtocolService } from '@services/Protocol';
import { Storage } from '@utils/StorageArray';
export declare class MissingLiveness extends Error {
}
export declare class ProtocolOptIn {
    private readonly storage;
    private readonly protocol;
    postOptInCallbacks: Array<(mnemonic: string) => Promise<void>>;
    private completedLivenessOverride;
    constructor(storage: Storage, protocol: ProtocolService);
    isOptedIn(): Promise<boolean>;
    private mnemonicKey;
    hasCompletedLiveness(): Promise<boolean>;
    getMnemonic(): Promise<string | undefined>;
    optIn(mnemonic: string): Promise<void>;
    private runCallbacks;
    checkOptIn(): Promise<void>;
    private tryRegisterIdentity;
}
