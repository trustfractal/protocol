import { ApiPromise } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import type { KeyringPair } from '@polkadot/keyring/types';
import { u64 } from '@polkadot/types';
import { BlockHash } from '@polkadot/types/interfaces';
import type { AccountData } from '@polkadot/types/interfaces';
import { DataHost } from '@services/DataHost';
import { TxnWatcher } from '@trustfractal/polkadot-utils';
import { Storage } from '@utils/StorageArray';

export class IdentityRegistrationFailed extends Error {
  constructor(message?: string) {
    super(message);
    this.name = 'IdentityRegistrationFailed';
  }
}

export class CannotExtendDataset extends Error {
  constructor(message?: string) {
    super(message);
    this.name = 'CannotExtendDataset';
  }
}

export class MintingRegistrationFailed extends Error {
  constructor(message?: string) {
    super(message);
    this.name = 'MintingRegistrationFailed';
  }
}

export class CannotExtend extends Error {
  constructor(public readonly proof: string | null) {
    super(`Cannot extend existing proof`);
    this.name = 'CannotExtend';
  }
}

const MINTING_PERIOD_LENGTH = 14400;

export class ProtocolService {
  private fractalIdCache: u64 | null = null;

  constructor(
    private readonly api: Promise<ApiPromise>,
    public signer: KeyringPair | null,
    private readonly dataHost: DataHost
  ) {}

  public async registerForMinting(): Promise<string | undefined> {
    const latestProof = await this.latestExtensionProof();
    console.log(`Latest proof from chain ${latestProof}`);

    const extensionProof = await this.dataHost.extensionProof(latestProof);
    if (extensionProof == null)
      throw new CannotExtendDataset(
        `Current tree: ${await this.dataHost.currentTree()}`
      );

    const hash = await this.submitMintingExtrinsic(extensionProof);
    if (!(await this.isRegisteredForMinting())) {
      throw new MintingRegistrationFailed();
    }
    return hash;
  }

  private async latestExtensionProof(): Promise<string | null> {
    const fractalId = await this.registeredFractalId();
    if (fractalId == null) return null;

    // Blue-green strategy handling migration of blockchain storage.
    try {
      // Will be long-term code.
      const dataset = await (
        await this.api
      ).query.fractalMinting.accountIdDatasets(this.address(), fractalId);
      return dataset.toHuman() as string | null;
    } catch (e) {
      // TODO(shelbyd): Delete this after rollout of storage change.
      const dataset = await (
        await this.api
      ).query.fractalMinting.idDatasets(fractalId);
      return dataset.toHuman() as string | null;
    }
  }

  private async submitMintingExtrinsic(proof: string): Promise<string> {
    console.log(`Submitting proof ${proof}`);
    const txn = (await this.api).tx.fractalMinting.registerForMinting(
      null,
      proof
    );

    const { hash } = await TxnWatcher.signAndSend(
      txn as any,
      this.requireSigner()
    ).inBlock();
    return hash;
  }

  private requireSigner(): KeyringPair {
    if (this.signer == null) {
      throw new Error('Method requires signer to be defined');
    } else {
      return this.signer;
    }
  }

  public async isRegisteredForMinting(id?: number): Promise<boolean> {
    const fractalId = await this.registeredFractalId();
    if (fractalId == null) return false;

    const api = await this.api;
    const rewards = api.query.fractalMinting.nextMintingRewards;
    const storage =
      id == null
        ? (await rewards(fractalId))!
        : (await rewards.at(await this.hash(id), fractalId))!;
    return !storage.isEmpty;
  }

  private async registeredFractalId(): Promise<u64 | null> {
    if (this.fractalIdCache != null) return this.fractalIdCache;

    const keys = await this.withApi((api) =>
      api.query.fractalMinting.accountIds.keys(this.address())
    );
    if (keys.length !== 1) return null;

    const fractalId = keys[0].args[1] as u64;
    this.fractalIdCache = fractalId;
    return fractalId;
  }

  public async ensureIdentityRegistered(): Promise<void> {
    if (await this.isIdentityRegistered()) return;

    console.log('Identity is not registered, trying to register');
    if (await this.isIdentityRegistered()) {
      console.log('Identity successfully registered');
    } else {
      throw new IdentityRegistrationFailed();
    }
  }

  public async isIdentityRegistered(): Promise<boolean> {
    const fractalId = await this.registeredFractalId();
    return fractalId != null;
  }

  public address() {
    return this.requireSigner().address;
  }

  public async getBalance(address: string): Promise<AccountData> {
    return await this.withApi(async (api) => {
      return (await api.query.system.account(address)).data;
    });
  }

  public async watchBalance(cb: (accountData: AccountData) => void) {
    const unsub = await this.withApi((api) => {
      return api.query.system.account(this.address(), ({ data }) => {
        cb(data);
      });
    });
    return unsub;
  }

  public addressForMnemonic(mnemonic: string): string {
    const keyring = new Keyring({ type: 'sr25519' });
    const signer = keyring.addFromUri(mnemonic);
    return signer.address;
  }

  async saveSigner(storage: Storage) {
    await storage.setItem(
      'protocol/signer',
      JSON.stringify(this.requireSigner().toJson())
    );
  }

  static signerFromMnemonic(mnemonic: string) {
    const keyring = new Keyring({ type: 'sr25519' });
    return keyring.addFromUri(mnemonic);
  }

  static async saveSignerMnemonic(storage: Storage, mnemonic: string) {
    await storage.setItem(
      'protocol/signer',
      JSON.stringify(ProtocolService.signerFromMnemonic(mnemonic).toJson())
    );
  }

  static async signerFromStorage(storage: Storage) {
    const maybeSigner = await storage.getItem('protocol/signer');
    if (maybeSigner == null)
      throw new Error('No signer in the provided storage');
    const parsedSigner = JSON.parse(maybeSigner);

    const keyring = new Keyring({ type: 'sr25519' });
    const signer = keyring.addFromJson(parsedSigner);
    signer.unlock();
    return signer;
  }

  async sendToAddress(
    address: string,
    amount: number | bigint
  ): Promise<string> {
    const api = await this.api;
    const txn = api.tx.balances.transfer(address, amount);
    const watcher = TxnWatcher.signAndSend(txn as any, this.requireSigner());
    const { hash } = await watcher.inBlock();
    return hash;
  }

  createSigner(mnemonic: string) {
    const keyring = new Keyring({ type: 'sr25519' });
    const signer = keyring.addFromUri(mnemonic);
    return signer;
  }

  async sweepFromMnemonic(mnemonic: string): Promise<string> {
    return await this.withApi(async (api) => {
      const signer = this.createSigner(mnemonic);
      const balance = (await this.getBalance(signer.address)).free.toBigInt();

      const fee = (
        await api.tx.balances
          .transfer(this.address(), balance)
          .paymentInfo(signer)
      ).partialFee;

      const txn = api.tx.balances.transfer(
        this.address(),
        balance - fee.toBigInt()
      );
      const { hash } = await TxnWatcher.signAndSend(txn as any, signer, {
        fee,
      } as any).inBlock();
      return hash;
    });
  }

  // `MintingHistoryEvent`s in order from most recent to oldest. Capped at
  // `numEvents` and last 7 periods worth of blocks.
  async mintingHistory(numEvents: number): Promise<Array<MintingHistoryEvent>> {
    const latestHeader = await this.withApi((api) => api.rpc.chain.getHeader());
    const latestNumber = latestHeader.number.toNumber();

    const promises = Array.from({ length: 7 }, (_, i) => i).map((i) =>
      this.mintingEventForPeriod(latestNumber - i * MINTING_PERIOD_LENGTH)
    );
    const result: Array<MintingHistoryEvent> = [];
    while (result.length < numEvents) {
      const firstPromise = promises.shift();
      if (firstPromise == null) break;

      const event = await firstPromise;
      if (event != null) result.push(event);
    }
    return result.slice(0, numEvents);
  }

  // Returns the most relevant minting event for the user that occured in the
  // period containing the provided block.
  private async mintingEventForPeriod(
    blockNum: number
  ): Promise<MintingHistoryEvent | null> {
    if (blockNum <= 0) return null;

    const periodNumber = Math.floor(blockNum / MINTING_PERIOD_LENGTH);
    const beginningOfPeriod = periodNumber * MINTING_PERIOD_LENGTH + 1;
    const endOfPeriod = (periodNumber + 1) * MINTING_PERIOD_LENGTH;

    const received = await this.mintingReceived(endOfPeriod);
    if (received != null) return received;

    try {
      return await this.mintingRegistration(beginningOfPeriod, endOfPeriod);
    } catch (e) {
      if (e instanceof BlockNumberOutsideRange)
        return await this.mintingRegistration(beginningOfPeriod, blockNum);
      throw e;
    }
  }

  private async mintingReceived(end: number): Promise<MintingReceived | null> {
    // Since minting occurs in the on_finalize hook, users don't show up as
    // registered on the exact block minting occurs.
    try {
      const wasRegistered = await this.isRegisteredForMinting(end - 1);
      if (!wasRegistered) return null;
    } catch (e) {
      if (e instanceof BlockNumberOutsideRange) return null;
      throw e;
    }

    const endHash = await this.hash(end);
    const events = (
      await this.withApi((api) => api.query.system.events.at(endHash))
    ).map((e) => e.event);
    const minting = events.find(
      (e) => e.method === 'Minted' && e.section === 'fractalMinting'
    )!;

    // Testnet has the minting event with 2 arguments instead of the mainnet's
    // 4. The 2 argument case can be removed once the testnet is using the
    // latest runtime.
    const amount =
      minting.data.length === 2
        ? (minting.data[0] as any).toNumber() /
          (minting.data[1] as any).toNumber()
        : (minting.data[1] as any).toNumber();

    return {
      kind: 'received',
      at: await this.timestampForBlock(end),
      amount,
    };
  }

  private async mintingRegistration(
    start: number,
    end: number
  ): Promise<MintingRegistered | null> {
    const registeredAt = await binarySearch(start, end, async (n) => {
      return await this.isRegisteredForMinting(n);
    });

    if (registeredAt == null) return null;
    return {
      kind: 'registered',
      at: await this.timestampForBlock(registeredAt),
    };
  }

  private async withApi<T>(f: (api: ApiPromise) => T): Promise<T> {
    return await f(await this.api);
  }

  private async hash(n: number): Promise<BlockHash> {
    const hash = await this.withApi((api) => api.rpc.chain.getBlockHash(n));
    if (hash.isEmpty) throw new BlockNumberOutsideRange(n);
    return hash;
  }

  private async timestampForBlock(n: number): Promise<Date> {
    const hash = await this.hash(n);
    const timestamp = await this.withApi((api) =>
      api.query.timestamp.now.at(hash)
    );
    return new Date(timestamp.toNumber());
  }
}

class BlockNumberOutsideRange extends Error {
  constructor(n: number) {
    super(`Block number ${n} not found`);
    this.name = 'BlockNumberOutsideRange';
  }
}

// Returns the smallest integer where the provided function returns true or null
// if the `minTrue` argument is false or if the `maxFalse` argument is true.
async function binarySearch(
  maxFalse: number,
  minTrue: number,
  f: (n: number) => Promise<boolean>
): Promise<number | null> {
  if (!(await f(minTrue))) return null;
  if (await f(maxFalse)) return null;

  while (minTrue - maxFalse > 1) {
    const mid = Math.floor(minTrue / 2 + maxFalse / 2);
    if (await f(mid)) {
      minTrue = mid;
    } else {
      maxFalse = mid;
    }
  }
  return minTrue;
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
export type MintingHistoryEvent = MintingReceived | MintingRegistered;
