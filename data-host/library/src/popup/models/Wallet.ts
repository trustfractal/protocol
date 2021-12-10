import { u8aToHex } from '@polkadot/util';
import {
  encodeAddress,
  mnemonicGenerate,
  mnemonicToMiniSecret,
  sr25519PairFromSeed,
} from '@polkadot/util-crypto';

export default class Wallet {
  public static generate() {
    const mnemonic = mnemonicGenerate();

    return new Wallet(mnemonic);
  }

  public static fromMnemonic(mnemonic: string) {
    return new Wallet(mnemonic);
  }

  public mnemonic: string;
  public _seed: Uint8Array;
  public _publicKey: Uint8Array;
  public address: string;

  public constructor(mnemonic: string) {
    this.mnemonic = mnemonic;

    this._seed = mnemonicToMiniSecret(mnemonic);
    const { publicKey } = sr25519PairFromSeed(this._seed);
    this._publicKey = publicKey;
    this.address = encodeAddress(publicKey);
  }

  public get seed(): string {
    return u8aToHex(this._seed);
  }

  public get publicKey(): string {
    return u8aToHex(this._publicKey);
  }

  public serialize(): string {
    return this.mnemonic;
  }
}
