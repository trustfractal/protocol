import { ApiPromise, WsProvider } from '@polkadot/api';
import { environment } from '@popup/Environment';
import { StorageService } from '@popup/services/StorageService';
import { DataHost } from '@services/DataHost';
import { FractalAccountConnector } from '@services/FractalAccount';
import { ProtocolService } from '@services/Protocol';
import { ProtocolOptIn } from '@services/ProtocolOptIn';
import { AggregateMultiContext, MultiContext } from '@utils/MultiContext';
import { ValueCache } from '@utils/ReactHooks';

//TODO(melatron): We should import these from our source of truth, /blockchain/types.json.
const types = {
  FractalId: 'u64',
  MerkleTree: 'Raw',
};

let storageService: StorageService;
export function getStorageService() {
  if (storageService === undefined) {
    storageService = new StorageService();
  }
  return storageService;
}

let dataHost: DataHost;
export function getDataHost() {
  if (dataHost === undefined) {
    dataHost = new DataHost(storageService, storageService);
  }
  return dataHost;
}

async function getApi() {
  try {
    const url = 'wss://nodes.testnet.fractalprotocol.com'; //TODO(melatron): get the mainnet url
    const provider = new WsProvider(url);
    return await ApiPromise.create({ provider, types });
  } catch (e) {
    console.error(e);
    const provider = new WsProvider(environment.PROTOCOL_RPC_ENDPOINT);
    return await ApiPromise.create({ provider, types });
  }
}

let protocol: ProtocolService;
export function getProtocolService(mnemonic?: string) {
  if (protocol === undefined) {
    const signer = mnemonic
      ? ProtocolService.signerFromMnemonic(mnemonic)
      : null;
    protocol = new ProtocolService(getApi(), signer, getDataHost());

    getProtocolOptIn()
      .getMnemonic()
      .then(async (mnemonic) => {
        if (mnemonic) {
          getProtocolService().signer =
            ProtocolService.signerFromMnemonic(mnemonic);
        }
      });
  }

  return protocol;
}

let protocolOptIn: ProtocolOptIn;
export function getProtocolOptIn() {
  if (protocolOptIn === undefined) {
    protocolOptIn = new ProtocolOptIn(
      getStorageService(),
      getProtocolService()
    );

    protocolOptIn.postOptInCallbacks.push(async () => {
      await getDataHost().enable();
    });
    protocolOptIn.postOptInCallbacks.push(async (mnemonic: string) => {
      getProtocolService().signer =
        ProtocolService.signerFromMnemonic(mnemonic);
    });
  }
  return protocolOptIn;
}

let fractalAccountConnector: FractalAccountConnector;
export function getFractalAccountConnector() {
  if (fractalAccountConnector == null) {
    fractalAccountConnector = new FractalAccountConnector(getStorageService());
  }
  return fractalAccountConnector;
}

let multiContext: MultiContext;
export function getMultiContext() {
  if (multiContext == null) {
    multiContext = new AggregateMultiContext([getFractalAccountConnector()]);
  }
  return multiContext;
}

let valueCache: ValueCache;
export function getValueCache() {
  if (valueCache == null) {
    valueCache = new ValueCache(getStorageService());
  }
  return valueCache;
}
