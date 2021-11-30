import { Storage, StorageArray } from '@utils/StorageArray';

export class DataHost {
  private build!: (s: string) => string | undefined;
  private extend_multiple!: (mtree: string, leaves: any) => string | undefined;
  private prune_balanced!: (s: string) => string | undefined;
  private strict_extension_proof!: (
    mtree_a: string,
    mtree_b: string
  ) => string | undefined;

  constructor(
    private readonly metadata: Storage,
    private readonly sensitive: Storage
  ) {}

  private key(key: string) {
    return `data_host/${key}`;
  }

  async init() {
    const { build, extend_multiple, prune_balanced, strict_extension_proof } =
      await import('@vendor/merklex-js/merklex_js.js');

    this.build = build;
    this.extend_multiple = extend_multiple;
    this.prune_balanced = prune_balanced;
    this.strict_extension_proof = strict_extension_proof;
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
      return hexPrefix(this.prune_balanced(currentTree)!);
    } else {
      const previousTree = latestProof.includes('x')
        ? latestProof.split('x')[1]
        : latestProof;
      const proof = this.strict_extension_proof(currentTree, previousTree);
      return proof && hexPrefix(proof);
    }
  }

  async currentTree(): Promise<string | undefined> {
    const allItems = [];
    for await (const item of this.array().iter()) {
      allItems.push(item);
    }
    if (allItems.length === 0) return;

    return this.buildTree(allItems.map((i) => JSON.stringify(i)));
  }

  buildTree(items: string[]): string {
    const begin = this.build(items[0])!;
    return this.extend_multiple(begin, items.slice(1))!;
  }
}

function hexPrefix(s: string): string {
  return `0x${s}`;
}
