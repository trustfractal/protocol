import { Storage, StorageArray } from '@utils/StorageArray';

describe('StorageArray', () => {
  const prefix = 'some_prefix';

  class MockStorage implements Storage {
    private readonly data = new Map();

    getItem(key: string) {
      if (typeof key !== 'string') {
        throw TypeError(`Called getItem with ${typeof key} instead of string`);
      }

      return Promise.resolve(this.data.get(key));
    }

    setItem(key: string, value: string) {
      if (typeof value !== 'string') {
        throw TypeError(
          `Called setItem with ${typeof value} instead of string`
        );
      }
      if (!key.startsWith(prefix)) {
        throw `Key ${key} does not start with prefix ${prefix}`;
      }

      this.data.set(key, value);
      return Promise.resolve();
    }

    hasItem(key: string) {
      return Promise.resolve(this.data.has(key));
    }

    removeItem(key: string) {
      this.data.delete(key);
      return Promise.resolve();
    }
  }

  it('has length zero with no data', async () => {
    const mockStorage = new MockStorage();
    const subject = new StorageArray(mockStorage, prefix);

    expect(await subject.length()).toEqual(0);
  });

  it('has length 1 after insert', async () => {
    const mockStorage = new MockStorage();
    const subject = new StorageArray(mockStorage, prefix);

    await subject.push('foo');

    expect(await subject.length()).toEqual(1);
  });

  it('has item at index', async () => {
    const mockStorage = new MockStorage();
    const subject = new StorageArray(mockStorage, prefix);

    await subject.push('foo');

    expect(await subject.get(0)).toEqual('foo');
  });

  it('stringifies item', async () => {
    const mockStorage = new MockStorage();
    const subject = new StorageArray(mockStorage, prefix);

    await subject.push({ foo: 42 });

    expect(await subject.get(0)).toEqual({ foo: 42 });
  });

  it('has 3 items after pushes', async () => {
    const mockStorage = new MockStorage();
    const subject = new StorageArray(mockStorage, prefix);

    await subject.push('foo');
    await subject.push('bar');
    await subject.push('baz');

    expect(await subject.length()).toEqual(3);
  });

  it('iterates over items', async () => {
    const mockStorage = new MockStorage();
    const subject = new StorageArray(mockStorage, prefix);

    await subject.push('foo');
    await subject.push('bar');
    await subject.push('baz');

    const items = [];
    for await (const item of subject.iter()) {
      items.push(item);
    }

    expect(items).toEqual(['foo', 'bar', 'baz']);
  });

  it('iterates over items in a reverse order', async () => {
    const mockStorage = new MockStorage();
    const subject = new StorageArray(mockStorage, prefix);

    await subject.push('foo');
    await subject.push('bar');
    await subject.push('baz');

    const items = [];
    for await (const item of subject.iterBack()) {
      items.push(item);
    }

    expect(items).toEqual(['baz', 'bar', 'foo']);
  });

  it('multiple instances do not race each other', async () => {
    const mockStorage = new MockStorage();
    const inParallel = 100;

    const promises = [];
    for (const i of [...Array(inParallel).keys()]) {
      promises[i] = new StorageArray(mockStorage, prefix).push(i);
    }

    await Promise.all(promises);

    const subject = new StorageArray(mockStorage, prefix);
    expect(await subject.length()).toEqual(inParallel);

    const itemExists = [...Array(inParallel).keys()].map(() => false);
    for await (const item of subject.iter()) {
      itemExists[item] = true;
    }
    for (const exists of itemExists) {
      expect(exists).toEqual(true);
    }
  });

  it('returns the index with push', async () => {
    const mockStorage = new MockStorage();
    const subject = new StorageArray(mockStorage, prefix);

    await subject.push('foo');
    await subject.push('bar');
    await subject.push('baz');

    expect(await subject.push('qux')).toEqual(3);
  });
});
