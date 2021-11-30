import { MultiContext } from '@utils/MultiContext';
import { Storage } from '@utils/StorageArray';
import { ReplaySubject } from 'rxjs';
interface Tokens {
    scopes: string;
}
export declare class NotConnectedError extends Error {
    constructor(message?: string);
}
export declare class FractalAccountConnector extends MultiContext {
    private readonly storage;
    tokens: Tokens | null;
    connectedAccount$: ReplaySubject<boolean>;
    constructor(storage: Storage);
    hasConnectedAccount(): boolean;
    doConnect(urlOverride?: string): Promise<void>;
    willAcceptNextTokens(): Promise<boolean>;
    setTokens(tokens: Tokens): Promise<void>;
    getTokens(): Promise<any>;
    private requireTokens;
    getCatfishToken(): Promise<any>;
    getScopes(): Promise<any>;
    clearTokens(): Promise<void>;
}
export {};
