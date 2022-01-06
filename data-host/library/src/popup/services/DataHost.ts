import { Storage, StorageArray } from '@utils/StorageArray';
import {
  build,
  extend_multiple,
  prune_balanced,
  strict_extension_proof,
} from '@vendor/merklex-js/merklex_js.js';

export class DataHost {
  constructor(
    private readonly metadata: Storage,
    private readonly sensitive: Storage
  ) {}

  private key(key: string) {
    return `data_host/${key}`;
  }

  async enable() {
    await this.metadata.setItem(this.key('enabled'), 'true');
  }

  async disable() {
    await this.metadata.setItem(this.key('enabled'), 'false');
  }

  async isEnabled(): Promise<boolean> {
    const value = await this.metadata.getItem(this.key('enabled'));
    return value === 'true';
  }

  async storeFact(fact: any) {
    if (!(await this.isEnabled())) return;

    this.array().push(fact);
  }

  private array() {
    return new StorageArray(this.sensitive, this.key('facts'));
  }

  public iter() {
    return this.array().iter();
  }

  public iterBack() {
    return this.array().iterBack();
  }

  async extensionProof(
    latestProof: string | null
  ): Promise<string | undefined> {
    const currentTree = await this.currentTree();
    if (currentTree == null) return;

    if (latestProof == null) {
      return hexPrefix(prune_balanced(currentTree)!);
    } else {
      const previousTree = latestProof.includes('x')
        ? latestProof.split('x')[1]
        : latestProof;
      const proof = strict_extension_proof(currentTree, previousTree);
      return proof && hexPrefix(proof);
    }
  }

  async currentTree(): Promise<string | undefined> {
    const allItems = [];
    for await (const item of this.array().iter()) {
      allItems.push(item);
    }
    if (allItems.length === 0) return;

    return buildTree(allItems.map((i) => JSON.stringify(i)));
  }
}

function buildTree(items: string[]): string {
  const begin = build(items[0])!;
  return extend_multiple(begin, items.slice(1))!;
}

function hexPrefix(s: string): string {
  return `0x${s}`;
}
