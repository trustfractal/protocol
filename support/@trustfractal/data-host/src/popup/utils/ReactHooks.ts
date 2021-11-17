import { Storage } from '@utils/StorageArray';
import { useCallback, useEffect, useState } from 'react';
import { Observable } from 'rxjs';

// Note that this memoizes the loader promise since the common case is
// explicitly not wanting it to change between calls. This makes the API simpler
// while being slightly potentially surprising if the callback is expected to be
// evaluated multiple times (which we don't expect to be common).
export function useLoadedState<T>(
  loader: () => Promise<T>,
  watchedVars: unknown[] = []
): Load<T> {
  // We keep the state that's bound together in the same useState so React
  // doesn't trigger renders when setting one but not the other.
  const [[loaded, value], setLoadedValue] = useState<[true, T] | [false, null]>(
    [false, null]
  );

  // The common case is to only want to call the loader once, so we memoize it
  // for the user to prevent all users from having to do that themselves.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const memoLoader = useCallback(() => loader(), [...watchedVars]);

  useEffect(() => {
    let active = true;
    (async () => {
      if (loaded) return;
      if (!active) return;

      const v = await memoLoader();
      if (!active) return;

      setLoadedValue([true, v]);
    })();
    return () => {
      active = false;
    };
  }, [memoLoader, loaded]);

  const setValue = (t: T) => setLoadedValue([true, t]);
  const reload = () => setLoadedValue([false, null]);

  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(reload, [...watchedVars]);

  if (loaded) {
    return new Loaded<T>(value!, setValue, reload);
  } else {
    return new Loading(setValue, reload);
  }
}

export type Load<T, Orig = T> = Loading<T, Orig> | Loaded<T, Orig>;

export class Loading<T, Orig = T> {
  isLoaded: false = false;

  constructor(
    public readonly setValue: (t: Orig) => void,
    public readonly reload: () => void
  ) {}

  unwrapOrDefault<U>(def: U): U {
    return def;
  }

  map<U>(_fn: (t: T) => U): Loading<U, Orig> {
    return new Loading(this.setValue, this.reload);
  }
}

export class Loaded<T, Orig = T> {
  isLoaded: true = true;

  constructor(
    public readonly value: T,
    public readonly setValue: (t: Orig) => void,
    public readonly reload: () => void
  ) {}

  unwrapOrDefault<U>(_def: U): T {
    return this.value;
  }

  map<U>(fn: (t: T) => U): Loaded<U, Orig> {
    return new Loaded(fn(this.value), this.setValue, this.reload);
  }
}

export function useObservedState<T>(
  observable: () => Observable<T>
): Observed<T> {
  const [hasValue, setHasValue] = useState(false);
  const [value, setValue] = useState<T>();

  useEffect(() => {
    const sub = observable().subscribe((v) => {
      setValue(v);
      setHasValue(true);
    });
    return () => sub.unsubscribe();
  }, [observable]);

  if (hasValue) {
    return new Value(value!);
  } else {
    return new NotEmitted<T>();
  }
}

export interface Observed<T> {
  hasValue: boolean;
  value?: T;
  unwrapOrDefault<U>(def: U): T | U;
}

class NotEmitted<T> implements Observed<T> {
  hasValue = false;

  unwrapOrDefault<U>(def: U) {
    return def;
  }
}

class Value<T> {
  hasValue = true;

  constructor(public readonly value: T) {}

  unwrapOrDefault<U>(_: U) {
    return this.value;
  }
}

export interface CacheArgs<T> {
  cache: ValueCache;
  key: string;
  useFor?: number; // in seconds
  loader: () => Promise<T>;
  cacheWhen?: (t: T) => boolean;
  onValue?: (t: T) => void;
  // Default value: JSON.stringify.
  serialize?: (t: T) => string;
  // Default value: JSON.parse.
  deserialize?: (s: string) => T;
}

// React Hook that will as quickly as possible return a value from cache and
// continue to load the value from the source of truth once the useFor window
// has passed.
export function useCachedState<T>(args: CacheArgs<T>): Load<T> {
  // We keep the state that's bound together in the same useState so React
  // doesn't trigger renders when setting one but not the other.
  const [[loaded, value], setLoadedValue] = useState<[true, T] | [false, null]>(
    [false, null]
  );

  const setValue = (v: T) => {
    setLoadedValue([true, v]);

    if (args.onValue != null) args.onValue(v);

    const shouldCache = args.cacheWhen ? args.cacheWhen(v) : true;
    if (shouldCache) args.cache.set(args.key, serialize(v));
  };

  const serialize = args.serialize || JSON.stringify;
  const deserialize = args.deserialize || JSON.parse;

  useEffect(
    () => {
      if (loaded) return;

      let active = true;

      const setIfActive = (v: T) => {
        if (!active) return;
        setValue(v);
      };

      (async () => {
        const fromCache = await args.cache.get(args.key);
        if (fromCache != null) {
          const [setAt, serialized] = fromCache;

          const value = deserialize(serialized);
          setIfActive(value);
          if (args.useFor != null) {
            const waitFor = setAt + args.useFor * 1000 - new Date().getTime();
            await new Promise((resolve) =>
              setTimeout(resolve, Math.max(0, waitFor))
            );
          }
        }

        if (!active) return;

        const loadedValue = await args.loader();
        setIfActive(loadedValue);
      })();

      return () => {
        active = false;
      };
    },

    // The purpose of this hook is to minimize work. Providing no watched
    // arguments effectively memoizes the input object so the user doesn't
    // have to.
    // eslint-disable-next-line
    []
  );

  const reload = async () => {
    await args.cache.remove(args.key);
    setLoadedValue([false, null]);
  };

  if (loaded) {
    return new Loaded(value as T, setValue, reload);
  } else {
    const immediateCache = args.cache.getImmediate(args.key);
    if (immediateCache == null) {
      return new Loading(setValue, reload);
    } else {
      const deserialized = deserialize(immediateCache[1]);
      // Use setTimeout since we are not allowed to set a state value during
      // a render.
      setTimeout(() => setValue(deserialized));
      return new Loaded(deserialized, setValue, reload);
    }
  }
}

export class ValueCache {
  private memory = new Map<string, string>();

  constructor(private readonly storage: Storage) {
    // We could load all values into memory on construction and wait at the top
    // level of the app for the loading to finish. That way we can show all the
    // cached values immediately and not flash loading screens for the ~40ms it
    // takes to load from storage.
    //
    // This would require:
    //   1. Cleaning up old cache values. Probably having to store when the
    //      item expires instead of when it was stored.
    //   2. Making a "StorageMap" class that supports iterating over all
    //      entries in the map without iterating over all keys in the storage.
    //   3. Actually waiting for the loading to finish at the top level of the
    //      app.
  }

  async get(key: string): Promise<[number, string] | null> {
    const immediate = this.getImmediate(key);
    if (immediate != null) return immediate;

    const s = await this.storage.getItem(`$value-cache/${key}`);
    if (s == null) return null;

    this.memory.set(key, s);
    return JSON.parse(s);
  }

  getImmediate(key: string): [number, string] | null {
    const fromMemory = this.memory.get(key);
    return fromMemory && JSON.parse(fromMemory);
  }

  async set(key: string, value: string): Promise<void> {
    const now = new Date().getTime();
    const toStore = JSON.stringify([now, value]);

    this.memory.set(key, toStore);
    await this.storage.setItem(`$value-cache/${key}`, toStore);
  }

  async remove(key: string): Promise<void> {
    await this.storage.removeItem(`$value-cache/${key}`);
    this.memory.delete(key);
  }
}
