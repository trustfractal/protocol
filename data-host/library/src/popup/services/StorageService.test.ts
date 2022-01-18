import { StorageService } from '@services/StorageService';
import {
  StorageServiceClear as ERROR_CLEAR,
  StorageServiceGetItem as ERROR_GET_ITEM,
  StorageServiceHasItem as ERROR_HAS_ITEM,
  StorageServiceRemoveItem as ERROR_REMOVE_ITEM,
  StorageServiceSetItem as ERROR_SET_ITEM,
} from '@services/StorageService';
import { chrome } from 'jest-chrome';

describe('Unit Storage Service', () => {
  console.error = jest.fn();
  describe('hasItem()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('Given an existing key for a stored value, hasItem returns true', async () => {
      // Prepare
      const key = 'key';
      const value = 'value';
      chrome.storage.local.get.mockImplementation(
        (_, callback: (items: { [key: string]: any }) => void) => {
          callback({ [key]: value });
        }
      );

      // Execute
      const result = await new StorageService().hasItem(key);

      // Assert
      const expectedResult = true;
      expect(result).toBe(expectedResult);
      expect(chrome.storage.local.get).toHaveBeenCalled();
    });

    it('Given an nonexistent key, hasItem returns false', async () => {
      // Prepare
      const key = 'key';
      chrome.storage.local.get.mockImplementation(
        (_, callback: (items: { [key: string]: any }) => void) => {
          callback({});
        }
      );

      // Execute
      const result = await new StorageService().hasItem(key);

      // Assert
      const expectedResult = false;
      expect(result).toBe(expectedResult);
      expect(chrome.storage.local.get).toHaveBeenCalled();
    });

    it('When a chrome error occurs, hasItem rejects with the error', async () => {
      // Prepare
      const key = 'key';
      const lastErrorMessage = 'Chrome could not get the item';
      const lastErrorGetter = jest.fn(() => lastErrorMessage);
      const lastError = {
        get message() {
          return lastErrorGetter();
        },
      };
      chrome.storage.local.get.mockImplementation(
        (_, callback: (items: { [key: string]: any }) => void) => {
          chrome.runtime.lastError = lastError;
          callback({});
          delete chrome.runtime.lastError;
        }
      );

      // Execute and Assert
      await expect(new StorageService().hasItem(key)).rejects.toThrow(
        new ERROR_HAS_ITEM(
          lastError,
          `StorageService: could not check if key '${key}' is set`
        )
      );
    });
  });
  describe('getItem()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('Given an existing key for a stored value, getItem returns the stored value', async () => {
      // Prepare
      const key = 'key';
      const value = 'value';
      chrome.storage.local.get.mockImplementation(
        (_, callback: (items: { [key: string]: any }) => void) => {
          callback({
            [key]: value,
          });
        }
      );

      // Execute
      const result = await new StorageService().getItem(key);

      // Assert
      const expectedResult = value;
      expect(result).toBe(expectedResult);
      expect(chrome.storage.local.get).toHaveBeenCalled();
    });

    it('Given an nonexistent key, getItem returns undefined', async () => {
      // Prepare
      const key = 'key';
      chrome.storage.local.get.mockImplementation(
        (_, callback: (items: { [key: string]: any }) => void) => {
          callback({});
        }
      );

      // Execute
      const result = await new StorageService().getItem(key);

      // Assert
      const expectedResult = undefined;
      expect(result).toBe(expectedResult);
      expect(chrome.storage.local.get).toHaveBeenCalled();
    });

    it('Given an nonexistent key and a default value, getItem returns the default value', async () => {
      // Prepare
      const key = 'key';
      const defaultValue = 'default_value';
      chrome.storage.local.get.mockImplementation(
        (_, callback: (items: { [key: string]: any }) => void) => {
          callback({});
        }
      );

      // Execute
      const result = await new StorageService().getItem(key, defaultValue);

      // Assert
      const expectedResult = defaultValue;
      expect(result).toBe(expectedResult);
      expect(chrome.storage.local.get).toHaveBeenCalled();
    });

    it('When a chrome error occurs, getItem rejects with the error', async () => {
      // Prepare
      const key = 'key';
      const lastErrorMessage = 'Chrome could not get the item';
      const lastErrorGetter = jest.fn(() => lastErrorMessage);
      const lastError = {
        get message() {
          return lastErrorGetter();
        },
      };
      chrome.storage.local.get.mockImplementation(
        (_, callback: (items: { [key: string]: any }) => void) => {
          chrome.runtime.lastError = lastError;
          callback({});
          delete chrome.runtime.lastError;
        }
      );

      // Execute and Assert
      await expect(new StorageService().getItem(key)).rejects.toThrow(
        new ERROR_GET_ITEM(
          lastError,
          `StorageService: could not get key '${key}'`
        )
      );
    });
  });
  describe('setItem()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('Given a key and a value, setItem stores the value', async () => {
      // Prepare
      const key = 'key';
      const value = 'value';
      chrome.storage.local.set.mockImplementation(
        // eslint-disable-next-line @typescript-eslint/ban-types
        (_: Object, callback?: () => void) => {
          callback?.();
        }
      );

      // Execute
      await new StorageService().setItem(key, value);

      // Assert
      expect(chrome.storage.local.set).toHaveBeenCalled();
    });

    it('When a chrome error occurs, setItem rejects with the error', async () => {
      // Prepare
      const key = 'key';
      const value = 'value';
      const lastErrorMessage = 'Chrome could not set the item';
      const lastErrorGetter = jest.fn(() => lastErrorMessage);
      const lastError = {
        get message() {
          return lastErrorGetter();
        },
      };
      chrome.storage.local.set.mockImplementation(
        // eslint-disable-next-line @typescript-eslint/ban-types
        (_: Object, callback?: () => void) => {
          chrome.runtime.lastError = lastError;
          callback?.();
          delete chrome.runtime.lastError;
        }
      );

      // Execute and Assert
      await expect(new StorageService().setItem(key, value)).rejects.toThrow(
        new ERROR_SET_ITEM(
          lastError,
          `StorageService: could not set value '${value}' for key '${key}'`
        )
      );
    });
  });
  describe('removeItem()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('Given an existing key, removeItem deletes the value', async () => {
      // Prepare
      const key = 'key';
      chrome.storage.local.remove.mockImplementation(
        (_keys: string | string[], callback?: () => void) => {
          callback?.();
        }
      );

      // Execute
      await new StorageService().removeItem(key);

      // Assert
      expect(chrome.storage.local.remove).toHaveBeenCalled();
    });

    it('When a chrome error occurs, removeItem rejects with the error', async () => {
      // Prepare
      const key = 'key';
      const lastErrorMessage = 'Chrome could not remove the item';
      const lastErrorGetter = jest.fn(() => lastErrorMessage);
      const lastError = {
        get message() {
          return lastErrorGetter();
        },
      };
      chrome.storage.local.remove.mockImplementation(
        (_keys: string | string[], callback?: () => void) => {
          chrome.runtime.lastError = lastError;
          callback?.();
          delete chrome.runtime.lastError;
        }
      );

      // Execute and Assert
      await expect(new StorageService().removeItem(key)).rejects.toThrow(
        new ERROR_REMOVE_ITEM(
          lastError,
          `StorageService: could not remove key '${key}'`
        )
      );
    });
  });
  describe('clearStorage()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('For an not empty storage, clearStorage should delete all items', async () => {
      // Prepare
      chrome.storage.local.clear.mockImplementation((callback?: () => void) => {
        callback?.();
      });

      // Execute
      await new StorageService().clear();

      // Assert
      expect(chrome.storage.local.clear).toHaveBeenCalled();
    });
    it('When a chrome error occurs, clear rejects with the error', async () => {
      // Prepare
      const lastErrorMessage = 'Chrome could not clear';
      const lastErrorGetter = jest.fn(() => lastErrorMessage);
      const lastError = {
        get message() {
          return lastErrorGetter();
        },
      };
      chrome.storage.local.clear.mockImplementation((callback?: () => void) => {
        chrome.runtime.lastError = lastError;
        callback?.();
        delete chrome.runtime.lastError;
      });

      // Execute and Assert
      await expect(new StorageService().clear()).rejects.toThrow(
        new ERROR_CLEAR(lastError, `StorageService: could not clear`)
      );
    });
  });
});
