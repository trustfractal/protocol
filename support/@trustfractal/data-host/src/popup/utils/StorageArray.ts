export interface Storage {
  setItem(key: string, value: string): Promise<void>;
  getItem(key: string): Promise<string | undefined>;
  hasItem(key: string): Promise<boolean>;
  removeItem(key: string): Promise<void>;
}

class PrefixStorage {
  constructor(
    private readonly base: Storage,
    private readonly prefix: string
  ) {}

  key(key: string): string {
    return `${this.prefix}/${key}`;
  }

  async setItem(key: string, value: string) {
    await this.base.setItem(this.key(key), value);
  }

  async getItem(key: string): Promise<any> {
    return await this.base.getItem(this.key(key));
  }

  async hasItem(key: string): Promise<boolean> {
    return await this.base.hasItem(this.key(key));
  }

  async removeItem(key: string) {
    await this.base.removeItem(this.key(key));
  }
}

class StringifyStorage {
  constructor(private readonly base: Storage) {}

  async setItem(key: string, value: any) {
    await this.base.setItem(key, JSON.stringify(value));
  }

  async getItem(key: string): Promise<any> {
    const value = await this.base.getItem(key);
    if (value == null) return null;
    return JSON.parse(value);
  }

  async hasItem(key: string): Promise<boolean> {
    return this.base.hasItem(key);
  }

  async removeItem(key: string) {
    await this.base.removeItem(key);
  }
}

export class StorageArray {
  base: StringifyStorage;

  constructor(rootStorage: Storage, prefix: string) {
    this.base = new StringifyStorage(new PrefixStorage(rootStorage, prefix));
  }

  async push(item: any): Promise<number> {
    return await withLock(this.base, '$lock', async () => {
      const length = await this.length();
      await this.base.setItem('length', length + 1);

      await this.base.setItem(length.toString(), item);
      return length;
    });
  }

  async length(): Promise<number> {
    const value = await this.base.getItem('length');
    return value || 0;
  }

  async get(index: number): Promise<any> {
    return this.base.getItem(index.toString());
  }

  iter() {
    const storage = this;
    return {
      [Symbol.asyncIterator]() {
        return {
          i: 0,
          async next() {
            if (this.i >= (await storage.length())) {
              return { done: true };
            }

            const value = await storage.get(this.i++);
            return { value, done: false };
          },
        };
      },
    };
  }

  iterBack() {
    const storage = this;
    return {
      [Symbol.asyncIterator]() {
        return {
          i: (async () => await storage.length())(),
          async next() {
            const i = (await this.i) - 1;

            if (i < 0) {
              return { done: true };
            }

            const value = await storage.get(i);
            this.i = (async () => (await this.i) - 1)();

            return { value, done: false };
          },
        };
      },
    };
  }
}

// Uses Lamport's first fast lock to acquire a lock on the entirety of the
// underlying storage.
//
// https://www.cs.rochester.edu/research/synchronization/pseudocode/fastlock.html
export async function withLock<T>(
  storage: Storage,
  key: string,
  callback: () => Promise<T>
) {
  const MAX_WAIT_MS = 2;

  const id = Math.random().toString().split('.')[1];
  const acquiring_lock = `${key}/acquiring_lock`;
  const holding_lock = `${key}/holding_lock`;

  do {
    await storage.setItem(acquiring_lock, id);

    if ((await storage.getItem(holding_lock)) != null) continue;
    await storage.setItem(holding_lock, id);

    if ((await storage.getItem(acquiring_lock)) !== id) {
      await new Promise((resolve) =>
        setTimeout(resolve, Math.random() * MAX_WAIT_MS)
      );
      if ((await storage.getItem(holding_lock)) !== id) continue;
    }
  } while (false);

  try {
    const result = await callback();
    await storage.removeItem(holding_lock);
    return result;
  } catch (e) {
    await storage.removeItem(holding_lock);
    throw e;
  }
}
