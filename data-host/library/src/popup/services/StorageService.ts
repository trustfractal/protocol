export class StorageServiceHasItem extends Error {
  public errorChrome: chrome.runtime.LastError;

  constructor(errorChrome: chrome.runtime.LastError, message?: string) {
    super(message);
    this.errorChrome = errorChrome;
    this.name = 'StorageServiceHasItem';
  }
}
export class StorageServiceGetItem extends Error {
  public errorChrome: chrome.runtime.LastError;

  constructor(errorChrome: chrome.runtime.LastError, message?: string) {
    super(message);
    this.errorChrome = errorChrome;
    this.name = 'StorageServiceGetItem';
  }
}
export class StorageServiceSetItem extends Error {
  public errorChrome: chrome.runtime.LastError;

  constructor(errorChrome: chrome.runtime.LastError, message?: string) {
    super(message);
    this.errorChrome = errorChrome;
    this.name = 'StorageServiceSetItem';
  }
}
export class StorageServiceRemoveItem extends Error {
  public errorChrome: chrome.runtime.LastError;

  constructor(errorChrome: chrome.runtime.LastError, message?: string) {
    super(message);
    this.errorChrome = errorChrome;
    this.name = 'StorageServiceRemoveItem';
  }
}
export class StorageServiceClear extends Error {
  public errorChrome: chrome.runtime.LastError;

  constructor(errorChrome: chrome.runtime.LastError, message?: string) {
    super(message);
    this.errorChrome = errorChrome;
    this.name = 'StorageServiceClear';
  }
}

export class StorageService {
  hasItem(key: string): Promise<boolean> {
    return new Promise((resolve, reject) => {
      chrome.storage.local.get([key], (items: { [key: string]: string }) => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(
            new StorageServiceHasItem(
              chrome.runtime.lastError,
              `StorageService: could not check if key '${key}' is set`
            )
          );
        }

        resolve(items[key] !== undefined);
      });
    });
  }

  getItem(key: string, ifNull?: string): Promise<string | undefined> {
    return new Promise((resolve, reject) => {
      chrome.storage.local.get([key], (items: { [key: string]: string }) => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(
            new StorageServiceGetItem(
              chrome.runtime.lastError,
              `StorageService: could not get key '${key}'`
            )
          );
        }

        resolve(items[key] || ifNull);
      });
    });
  }

  setItem(key: string, value: string): Promise<void> {
    return new Promise((resolve, reject) => {
      chrome.storage.local.set({ [key]: value }, () => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(
            new StorageServiceSetItem(
              chrome.runtime.lastError,
              `StorageService: could not set value '${value}' for key '${key}'`
            )
          );
        }

        resolve();
      });
    });
  }

  removeItem(key: string): Promise<void> {
    return new Promise((resolve, reject) => {
      chrome.storage.local.remove([key], () => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(
            new StorageServiceRemoveItem(
              chrome.runtime.lastError,
              `StorageService: could not remove key '${key}'`
            )
          );
        }

        resolve();
      });
    });
  }

  clear(): Promise<void> {
    return new Promise((resolve, reject) => {
      chrome.storage.local.clear(() => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(
            new StorageServiceClear(
              chrome.runtime.lastError,
              `StorageService: could not clear`
            )
          );
        }

        resolve();
      });
    });
  }
}
