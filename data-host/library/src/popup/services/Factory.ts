import { UserAlerts } from '@components/Alerts';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { environment } from '@popup/Environment';
import { StorageService } from '@popup/services/StorageService';
import { DataHost } from '@services/DataHost';
import { MintingRegistrar } from '@services/MintingRegistrar';
import { ProtocolService } from '@services/Protocol';
import { ProtocolOptIn } from '@services/ProtocolOptIn';
import { WindowsService } from '@services/WindowsService';
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

let mintingRegistrar: MintingRegistrar;
export function getMintingRegistrar() {
  if (mintingRegistrar === undefined) {
    const sleep = environment.IS_DEV ? 5 : 30 * 60;
    mintingRegistrar = new MintingRegistrar(storageService, sleep);
  }
  return mintingRegistrar;
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
    //TODO(melatron): The way signer is set feels really wrong. Needs to be refactored. Currently
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

let windows: WindowsService;
export function getWindowsService() {
  if (windows === undefined) {
    windows = new WindowsService();
  }
  return windows;
}

let protocolOptIn: ProtocolOptIn;
export function getProtocolOptIn() {
  if (protocolOptIn === undefined) {
    protocolOptIn = new ProtocolOptIn(
      getStorageService(),
      getProtocolService(),
      getWindowsService(),
      environment.LIVENESS_CHECK_URL
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

let userAlerts: UserAlerts;
export function getUserAlerts() {
  if (userAlerts == null) {
    userAlerts = new UserAlerts();
  }
  return userAlerts;
}

let valueCache: ValueCache;
export function getValueCache() {
  if (valueCache == null) {
    valueCache = new ValueCache(getStorageService());
  }
  return valueCache;
}
