import { DataHost } from '@services/DataHost';
import { FractalAccountConnector } from '@services/FractalAccount';
import { ProtocolService } from '@services/Protocol';
import { ProtocolOptIn } from '@services/ProtocolOptIn';
import { StorageService } from '@services/StorageService';
import { MultiContext } from '@utils/MultiContext';
import { ValueCache } from '@utils/ReactHooks';
export declare function getStorageService(): StorageService;
export declare function getDataHost(): DataHost;
export declare function getProtocolService(mnemonic?: string): ProtocolService;
export declare function getProtocolOptIn(): ProtocolOptIn;
export declare function getFractalAccountConnector(): FractalAccountConnector;
export declare function getMultiContext(): MultiContext;
export declare function getValueCache(): ValueCache;
