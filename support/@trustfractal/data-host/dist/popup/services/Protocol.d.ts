import { ApiPromise } from '@polkadot/api';
import type { KeyringPair } from '@polkadot/keyring/types';
import type { AccountData } from '@polkadot/types/interfaces';
import { DataHost } from '@services/DataHost';
import { Storage } from '@utils/StorageArray';
export declare class IdentityRegistrationFailed extends Error {
    constructor(message?: string);
}
export declare class CannotExtendDataset extends Error {
    constructor(message?: string);
}
export declare class MintingRegistrationFailed extends Error {
    constructor(message?: string);
}
export declare class CannotExtend extends Error {
    readonly proof: string | null;
    constructor(proof: string | null);
}
export declare class ProtocolService {
    private readonly api;
    signer: KeyringPair | null;
    private readonly dataHost;
    private fractalIdCache;
    constructor(api: Promise<ApiPromise>, signer: KeyringPair | null, dataHost: DataHost);
    registerForMinting(): Promise<string | undefined>;
    private latestExtensionProof;
    private submitMintingExtrinsic;
    private requireSigner;
    isRegisteredForMinting(id?: number): Promise<boolean>;
    private registeredFractalId;
    ensureIdentityRegistered(): Promise<void>;
    isIdentityRegistered(): Promise<boolean>;
    address(): string;
    getBalance(address: string): Promise<AccountData>;
    watchBalance(cb: (accountData: AccountData) => void): Promise<import("@polkadot/api/types").VoidFn>;
    addressForMnemonic(mnemonic: string): string;
    saveSigner(storage: Storage): Promise<void>;
    static signerFromMnemonic(mnemonic: string): KeyringPair;
    static saveSignerMnemonic(storage: Storage, mnemonic: string): Promise<void>;
    static signerFromStorage(storage: Storage): Promise<KeyringPair>;
    sendToAddress(address: string, amount: number | bigint): Promise<string>;
    createSigner(mnemonic: string): KeyringPair;
    sweepFromMnemonic(mnemonic: string): Promise<string>;
    mintingHistory(numEvents: number): Promise<Array<MintingHistoryEvent>>;
    private mintingEventForPeriod;
    private mintingReceived;
    private mintingRegistration;
    private withApi;
    private hash;
    private timestampForBlock;
}
export interface MintingReceived {
    kind: 'received';
    amount: number;
    at: Date;
}
export interface MintingRegistered {
    kind: 'registered';
    at: Date;
}
export declare type MintingHistoryEvent = MintingReceived | MintingRegistered;
