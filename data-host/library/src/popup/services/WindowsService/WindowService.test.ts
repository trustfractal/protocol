import { WindowsService } from '@services/WindowsService';
import {
  ERROR_CLOSE_WINDOW,
  ERROR_CREATE_TAB,
  ERROR_CREATE_WINDOW,
  ERROR_GET_ALL_WINDOWS,
  ERROR_GET_CURRENT_WINDOW,
  ERROR_GET_TAB,
  ERROR_GET_WINDOW,
  ERROR_QUERY_TABS,
  ERROR_UPDATE_TAB,
} from '@services/WindowsService/Errors';
import { chrome } from 'jest-chrome';
console.error = jest.fn();

describe('Unit Windows Service', () => {
  describe('createWindow()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('createWindow creates and returns a window', async () => {
      // Prepare
      const returnedWindow: chrome.windows.Window = {
        top: 0,
        height: 400,
        width: 600,
        state: 'normal',
        focused: true,
        alwaysOnTop: false,
        incognito: false,
        type: 'normal',
        id: 12,
        left: 0,
        sessionId: '11bf5b37-e0b8-42e0-8dcf-dc8c4aefc000',
      };
      chrome.windows.create.mockImplementation(
        (_, callback?: (window?: chrome.windows.Window) => void) => {
          callback?.(returnedWindow);
        }
      );

      // Execute
      const result = await new WindowsService().createWindow();

      // Assert
      const expectedResult = returnedWindow;
      expect(result).toBe(expectedResult);
      expect(chrome.windows.create).toHaveBeenCalled();
    });

    it('When a chrome error occurs, createWindow rejects with the error', async () => {
      // Prepare
      const returnedWindow: chrome.windows.Window = {
        top: 0,
        height: 400,
        width: 600,
        state: 'normal',
        focused: true,
        alwaysOnTop: false,
        incognito: false,
        type: 'normal',
        id: 12,
        left: 0,
        sessionId: '11bf5b37-e0b8-42e0-8dcf-dc8c4aefc000',
      };
      const lastErrorMessage = 'Chrome could not create window';
      const lastErrorGetter = jest.fn(() => lastErrorMessage);
      const lastError = {
        get message() {
          return lastErrorGetter();
        },
      };
      chrome.windows.create.mockImplementation(
        (_, callback?: (window?: chrome.windows.Window) => void) => {
          chrome.runtime.lastError = lastError;
          callback?.(returnedWindow);
          delete chrome.runtime.lastError;
        }
      );

      // Execute and Assert
      await expect(new WindowsService().createWindow()).rejects.toThrow(
        ERROR_CREATE_WINDOW(lastError)
      );
    });
  });
  describe('getCurrentWindow()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('getCurrentWindow returns the current window', async () => {
      // Prepare
      const returnedWindow: chrome.windows.Window = {
        top: 0,
        height: 400,
        width: 600,
        state: 'normal',
        focused: true,
        alwaysOnTop: false,
        incognito: false,
        type: 'normal',
        id: 12,
        left: 0,
        sessionId: '11bf5b37-e0b8-42e0-8dcf-dc8c4aefc000',
      };
      chrome.windows.getCurrent.mockImplementation(
        (_, callback: (window: chrome.windows.Window) => void) => {
          callback?.(returnedWindow);
        }
      );

      // Execute
      const result = await new WindowsService().getCurrentWindow();

      // Assert
      const expectedResult = returnedWindow;
      expect(result).toBe(expectedResult);
      expect(chrome.windows.getCurrent).toHaveBeenCalled();
    });

    it('When a chrome error occurs, getCurrentWindow rejects with the error', async () => {
      // Prepare
      const returnedWindow: chrome.windows.Window = {
        top: 0,
        height: 400,
        width: 600,
        state: 'normal',
        focused: true,
        alwaysOnTop: false,
        incognito: false,
        type: 'normal',
        id: 12,
        left: 0,
        sessionId: '11bf5b37-e0b8-42e0-8dcf-dc8c4aefc000',
      };
      const lastErrorMessage = 'Chrome could not get current window';
      const lastErrorGetter = jest.fn(() => lastErrorMessage);
      const lastError = {
        get message() {
          return lastErrorGetter();
        },
      };
      chrome.windows.getCurrent.mockImplementation(
        (_, callback: (window: chrome.windows.Window) => void) => {
          chrome.runtime.lastError = lastError;
          callback(returnedWindow);
          delete chrome.runtime.lastError;
        }
      );

      // Execute and Assert
      await expect(new WindowsService().getCurrentWindow()).rejects.toThrow(
        ERROR_GET_CURRENT_WINDOW(lastError)
      );
    });
  });
  describe('getAllWindows()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('getAllWindows returns the an array with all windows', async () => {
      // Prepare
      const returnedWindows: chrome.windows.Window[] = [
        {
          top: 0,
          height: 400,
          width: 600,
          state: 'normal',
          focused: true,
          alwaysOnTop: false,
          incognito: false,
          type: 'normal',
          id: 12,
          left: 0,
          sessionId: '11bf5b37-e0b8-42e0-8dcf-dc8c4aefc000',
        },
      ];
      chrome.windows.getAll.mockImplementation(
        (_, callback: (window: chrome.windows.Window[]) => void) => {
          callback?.(returnedWindows);
        }
      );

      // Execute
      const result = await new WindowsService().getAllWindows();

      // Assert
      const expectedResult = returnedWindows;
      expect(result).toBe(expectedResult);
      expect(chrome.windows.getAll).toHaveBeenCalled();
    });

    it('When a chrome error occurs, getCurrentWindow rejects with the error', async () => {
      // Prepare
      const returnedWindows: chrome.windows.Window[] = [
        {
          top: 0,
          height: 400,
          width: 600,
          state: 'normal',
          focused: true,
          alwaysOnTop: false,
          incognito: false,
          type: 'normal',
          id: 12,
          left: 0,
          sessionId: '11bf5b37-e0b8-42e0-8dcf-dc8c4aefc000',
        },
      ];
      const lastErrorMessage = 'Chrome could not get all windows';
      const lastErrorGetter = jest.fn(() => lastErrorMessage);
      const lastError = {
        get message() {
          return lastErrorGetter();
        },
      };
      chrome.windows.getAll.mockImplementation(
        (_, callback: (window: chrome.windows.Window[]) => void) => {
          chrome.runtime.lastError = lastError;
          callback(returnedWindows);
          delete chrome.runtime.lastError;
        }
      );

      // Execute and Assert
      await expect(new WindowsService().getAllWindows()).rejects.toThrow(
        ERROR_GET_ALL_WINDOWS(lastError)
      );
    });
  });
  describe('closeWindow()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('Given a window id, closeWindow closes the window', async () => {
      // Prepare
      const windowId = 12;
      chrome.windows.remove.mockImplementation(
        // eslint-disable-next-line @typescript-eslint/ban-types
        (removedWindowId, callback?: Function) => {
          chrome.windows.onRemoved.callListeners(removedWindowId);
          callback?.();
        }
      );

      // Execute
      await new WindowsService().closeWindow(windowId);

      // Assert
      expect(chrome.windows.remove).toHaveBeenCalled();
    });

    it('When a chrome error occurs, closeWindow rejects with the error', async () => {
      // Prepare
      const windowId = 12;
      const lastErrorMessage = 'Chrome could not close the window';
      const lastErrorGetter = jest.fn(() => lastErrorMessage);
      const lastError = {
        get message() {
          return lastErrorGetter();
        },
      };
      // eslint-disable-next-line @typescript-eslint/ban-types
      chrome.windows.remove.mockImplementation((_, callback?: Function) => {
        chrome.runtime.lastError = lastError;
        callback?.();
        delete chrome.runtime.lastError;
      });

      // Execute and Assert
      await expect(new WindowsService().closeWindow(windowId)).rejects.toThrow(
        ERROR_CLOSE_WINDOW(lastError)
      );
    });
  });
  describe('closeCurrentWindow()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('closeCurrentWindow closes the current window', async () => {
      // Prepare
      const currentWindow: chrome.windows.Window = {
        top: 0,
        height: 400,
        width: 600,
        state: 'normal',
        focused: true,
        alwaysOnTop: false,
        incognito: false,
        type: 'normal',
        id: 12,
        left: 0,
        sessionId: '11bf5b37-e0b8-42e0-8dcf-dc8c4aefc000',
      };
      chrome.windows.getCurrent.mockImplementation(
        (_, callback: (window: chrome.windows.Window) => void) => {
          callback?.(currentWindow);
        }
      );
      chrome.windows.remove.mockImplementation(
        // eslint-disable-next-line @typescript-eslint/ban-types
        (windowId, callback?: Function) => {
          chrome.windows.onRemoved.callListeners(windowId);
          callback?.();
        }
      );

      // Execute
      await new WindowsService().closeCurrentWindow();

      // Assert
      expect(chrome.windows.getCurrent).toHaveBeenCalled();
      expect(chrome.windows.remove).toHaveBeenCalled();
    });
  });
  describe('closeAllWindows()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('closeAllWindows closes all windows', async () => {
      const returnedWindows: chrome.windows.Window[] = [
        {
          top: 0,
          height: 400,
          width: 600,
          state: 'normal',
          focused: true,
          alwaysOnTop: false,
          incognito: false,
          type: 'normal',
          id: 12,
          left: 0,
          sessionId: '11bf5b37-e0b8-42e0-8dcf-dc8c4aefc000',
        },
      ];
      chrome.windows.getAll.mockImplementation(
        (_, callback: (window: chrome.windows.Window[]) => void) => {
          callback?.(returnedWindows);
        }
      );
      chrome.windows.remove.mockImplementation(
        // eslint-disable-next-line @typescript-eslint/ban-types
        (removedWindowId, callback?: Function) => {
          chrome.windows.onRemoved.callListeners(removedWindowId);
          callback?.();
        }
      );

      // Execute
      await new WindowsService().closeAllWindows();

      // Assert
      expect(chrome.windows.getAll).toHaveBeenCalled();
      expect(chrome.windows.remove).toHaveBeenCalled();
    });
  });
  describe('createTab()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('createTab creates and returns a tab', async () => {
      // Prepare
      const windowId = 12;
      const returnedTab: chrome.tabs.Tab = {
        index: 1,
        pinned: false,
        highlighted: false,
        windowId,
        active: true,
        incognito: false,
        selected: false,
        discarded: false,
        autoDiscardable: true,
        groupId: 123,
      };
      chrome.tabs.create.mockImplementation(
        (
          _createProperties: chrome.tabs.CreateProperties,
          callback?: (tab: chrome.tabs.Tab) => void
        ) => {
          callback?.(returnedTab);
        }
      );

      // Execute
      const result = await new WindowsService().createTab({
        url: 'http://test.unit',
      });

      // Assert
      const expectedResult = returnedTab;
      expect(result).toBe(expectedResult);
      expect(chrome.tabs.create).toHaveBeenCalled();
    });

    it('When a chrome error occurs, createTab rejects with the error', async () => {
      // Prepare
      const windowId = 12;
      const returnedTab: chrome.tabs.Tab = {
        index: 1,
        pinned: false,
        highlighted: false,
        windowId,
        active: true,
        incognito: false,
        selected: false,
        discarded: false,
        autoDiscardable: true,
        groupId: 123,
      };
      const lastErrorMessage = 'Chrome could not create tab';
      const lastErrorGetter = jest.fn(() => lastErrorMessage);
      const lastError = {
        get message() {
          return lastErrorGetter();
        },
      };
      chrome.tabs.create.mockImplementation(
        (
          _createProperties: chrome.tabs.CreateProperties,
          callback?: (tab: chrome.tabs.Tab) => void
        ) => {
          chrome.runtime.lastError = lastError;
          callback?.(returnedTab);
          delete chrome.runtime.lastError;
        }
      );

      // Execute and Assert
      await expect(
        new WindowsService().createTab({
          url: 'http://test.unit',
        })
      ).rejects.toThrow(ERROR_CREATE_TAB(lastError));
    });
  });
  describe('getAllPopups()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('Given an url, getAllPopups returns all popups window', async () => {
      // Prepare
      const returnedWindows: chrome.windows.Window[] = [
        {
          top: 0,
          height: 400,
          width: 600,
          state: 'normal',
          focused: true,
          alwaysOnTop: false,
          incognito: false,
          type: 'normal',
          id: 12,
          left: 0,
          sessionId: '11bf5b37-e0b8-42e0-8dcf-dc8c4aefc000',
        },
      ];
      chrome.windows.getAll.mockImplementation(
        (_, callback: (window: chrome.windows.Window[]) => void) => {
          callback?.(returnedWindows);
        }
      );

      // Execute
      const result = await new WindowsService().getAllWindows();

      // Assert
      const expectedResult = returnedWindows;
      expect(result).toBe(expectedResult);
      expect(chrome.windows.getAll).toHaveBeenCalled();
    });
  });
  describe('closeAllPopups()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('Given an url, closeAllPopups closes all popups window', async () => {
      // Prepare
      const returnedWindows: chrome.windows.Window[] = [
        {
          top: 0,
          height: 400,
          width: 600,
          state: 'normal',
          focused: true,
          alwaysOnTop: false,
          incognito: false,
          type: 'normal',
          id: 12,
          left: 0,
          sessionId: '11bf5b37-e0b8-42e0-8dcf-dc8c4aefc000',
        },
      ];
      chrome.windows.getAll.mockImplementation(
        (_, callback: (window: chrome.windows.Window[]) => void) => {
          callback?.(returnedWindows);
        }
      );
      chrome.windows.remove.mockImplementation(
        // eslint-disable-next-line @typescript-eslint/ban-types
        (windowId, callback?: Function) => {
          chrome.windows.onRemoved.callListeners(windowId);
          callback?.();
        }
      );

      // Execute
      await new WindowsService().closeAllPopups();

      // Assert
      expect(chrome.windows.getAll).toHaveBeenCalled();
      expect(chrome.windows.remove).toHaveBeenCalled();
    });
  });

  describe('getWindow()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('Given a window id, getWindow gets the corresponding window object', async () => {
      // Prepare
      const windowId = 12;
      const returnedWindow: chrome.windows.Window = {
        top: 0,
        height: 400,
        width: 600,
        state: 'normal',
        focused: true,
        alwaysOnTop: false,
        incognito: false,
        type: 'normal',
        id: windowId,
        left: 0,
        sessionId: '11bf5b37-e0b8-42e0-8dcf-dc8c4aefc000',
      };
      chrome.windows.get.mockImplementation(
        (
          _windowId: number,
          _getInfo: chrome.windows.GetInfo,
          callback: (window: chrome.windows.Window) => void
        ) => {
          callback?.(returnedWindow);
        }
      );

      // Execute
      const result = await new WindowsService().getWindow(windowId);

      // Assert
      const expectedResult = returnedWindow;
      expect(result).toBe(expectedResult);
      expect(chrome.windows.get).toHaveBeenCalled();
    });

    it('When a chrome error occurs, getWindow rejects with the error', async () => {
      // Prepare
      const windowId = 12;
      const returnedWindow: chrome.windows.Window = {
        top: 0,
        height: 400,
        width: 600,
        state: 'normal',
        focused: true,
        alwaysOnTop: false,
        incognito: false,
        type: 'normal',
        id: windowId,
        left: 0,
        sessionId: '11bf5b37-e0b8-42e0-8dcf-dc8c4aefc000',
      };
      const lastErrorMessage = 'Chrome could get the window';
      const lastErrorGetter = jest.fn(() => lastErrorMessage);
      const lastError = {
        get message() {
          return lastErrorGetter();
        },
      };
      chrome.windows.get.mockImplementation(
        (
          _windowId: number,
          _getInfo: chrome.windows.GetInfo,
          callback: (window: chrome.windows.Window) => void
        ) => {
          chrome.runtime.lastError = lastError;
          callback?.(returnedWindow);
          delete chrome.runtime.lastError;
        }
      );

      // Execute and Assert
      await expect(new WindowsService().getWindow(windowId)).rejects.toThrow(
        ERROR_GET_WINDOW(lastError, windowId)
      );
    });
  });

  describe('getTab()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it("Given a tab's window id, getTab gets the corresponding tab object", async () => {
      // Prepare
      const windowId = 12;
      const returnedTab: chrome.tabs.Tab = {
        index: 1,
        pinned: false,
        highlighted: false,
        windowId,
        active: true,
        incognito: false,
        selected: false,
        discarded: false,
        autoDiscardable: true,
        groupId: 123,
      };
      chrome.tabs.get.mockImplementation(
        (_tabId: number, callback: (tab: chrome.tabs.Tab) => void) => {
          callback?.(returnedTab);
        }
      );

      // Execute
      const result = await new WindowsService().getTab(windowId);

      // Assert
      const expectedResult = returnedTab;
      expect(result).toBe(expectedResult);
      expect(chrome.tabs.get).toHaveBeenCalled();
    });

    it('When a chrome error occurs, getTab rejects with the error', async () => {
      // Prepare
      const windowId = 12;
      const returnedTab: chrome.tabs.Tab = {
        index: 1,
        pinned: false,
        highlighted: false,
        windowId,
        active: true,
        incognito: false,
        selected: false,
        discarded: false,
        autoDiscardable: true,
        groupId: 123,
      };
      const lastErrorMessage = 'Chrome could get the tab';
      const lastErrorGetter = jest.fn(() => lastErrorMessage);
      const lastError = {
        get message() {
          return lastErrorGetter();
        },
      };
      chrome.tabs.get.mockImplementation(
        (_tabId: number, callback: (tab: chrome.tabs.Tab) => void) => {
          chrome.runtime.lastError = lastError;
          callback?.(returnedTab);
          delete chrome.runtime.lastError;
        }
      );

      // Execute and Assert
      await expect(new WindowsService().getTab(windowId)).rejects.toThrow(
        ERROR_GET_TAB(lastError, windowId)
      );
    });
  });

  describe('updateTab()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it("Given a tab's window id and config properties object, updateTab updates the given tab", async () => {
      // Prepare
      const windowId = 12;
      const configProperties = { pinned: true };
      const returnedTab: chrome.tabs.Tab = {
        index: 1,
        pinned: false,
        highlighted: false,
        windowId,
        active: true,
        incognito: false,
        selected: false,
        discarded: false,
        autoDiscardable: true,
        groupId: 123,
      };
      chrome.tabs.update.mockImplementation(
        (
          _tabId: number,
          _config: chrome.tabs.UpdateProperties,
          callback?: (tab?: chrome.tabs.Tab) => void
        ) => {
          callback?.(returnedTab);
        }
      );

      // Execute
      const result = await new WindowsService().updateTab(
        windowId,
        configProperties
      );

      // Assert
      const expectedResult = returnedTab;
      expect(result).toBe(expectedResult);
      expect(chrome.tabs.update).toHaveBeenCalled();
    });

    it('When a chrome error occurs, updateTab rejects with the error', async () => {
      // Prepare
      const windowId = 12;
      const configProperties = { pinned: true };
      const returnedTab: chrome.tabs.Tab = {
        index: 1,
        pinned: false,
        highlighted: false,
        windowId,
        active: true,
        incognito: false,
        selected: false,
        discarded: false,
        autoDiscardable: true,
        groupId: 123,
      };
      const lastErrorMessage = 'Chrome could update the tab';
      const lastErrorGetter = jest.fn(() => lastErrorMessage);
      const lastError = {
        get message() {
          return lastErrorGetter();
        },
      };
      chrome.tabs.update.mockImplementation(
        (
          _tabId: number,
          _config: chrome.tabs.UpdateProperties,
          callback?: (tab?: chrome.tabs.Tab) => void
        ) => {
          chrome.runtime.lastError = lastError;
          callback?.(returnedTab);
          delete chrome.runtime.lastError;
        }
      );

      // Execute and Assert
      await expect(
        new WindowsService().updateTab(windowId, configProperties)
      ).rejects.toThrow(ERROR_UPDATE_TAB(lastError, windowId));
    });
  });

  describe('queryTabs()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('Given a query tabs info, queryTabs gets the matched tabs', async () => {
      // Prepare
      const queryInfo = { active: true };
      const returnedTabs: chrome.tabs.Tab[] = [
        {
          index: 1,
          pinned: false,
          highlighted: false,
          windowId: 12,
          active: true,
          incognito: false,
          selected: false,
          discarded: false,
          autoDiscardable: true,
          groupId: 123,
        },
      ];

      chrome.tabs.query.mockImplementation(
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        (
          _queryInfo: chrome.tabs.QueryInfo,
          callback: (tabs: chrome.tabs.Tab[]) => void
        ) => {
          callback?.(returnedTabs);
        }
      );

      // Execute
      const result = await new WindowsService().queryTabs(queryInfo);

      // Assert
      const expectedResult = returnedTabs;
      expect(result).toBe(expectedResult);
      expect(chrome.tabs.query).toHaveBeenCalled();
    });

    it('When a chrome error occurs, queryTabs rejects with the error', async () => {
      // Prepare
      const queryInfo = { active: true };
      const returnedTabs: chrome.tabs.Tab[] = [
        {
          index: 1,
          pinned: false,
          highlighted: false,
          windowId: 12,
          active: true,
          incognito: false,
          selected: false,
          discarded: false,
          autoDiscardable: true,
          groupId: 123,
        },
      ];
      const lastErrorMessage = 'Chrome could get the window';
      const lastErrorGetter = jest.fn(() => lastErrorMessage);
      const lastError = {
        get message() {
          return lastErrorGetter();
        },
      };
      chrome.tabs.query.mockImplementation(
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        (
          _queryInfo: chrome.tabs.QueryInfo,
          callback: (tabs: chrome.tabs.Tab[]) => void
        ) => {
          chrome.runtime.lastError = lastError;
          callback?.(returnedTabs);
          delete chrome.runtime.lastError;
        }
      );

      // Execute and Assert
      await expect(new WindowsService().queryTabs(queryInfo)).rejects.toThrow(
        ERROR_QUERY_TABS(lastError)
      );
    });
  });

  describe('getActiveTabs()', () => {
    afterEach(() => {
      jest.resetAllMocks();
    });

    it('getActiveTabs gets the active tabs', async () => {
      // Prepare
      const windowId = 12;
      const returnedTabs: chrome.tabs.Tab[] = [
        {
          index: 1,
          pinned: false,
          highlighted: false,
          windowId: windowId,
          active: true,
          incognito: false,
          selected: false,
          discarded: false,
          autoDiscardable: true,
          groupId: 123,
        },
      ];
      const returnedWindow: chrome.windows.Window = {
        top: 0,
        height: 400,
        width: 600,
        state: 'normal',
        focused: true,
        alwaysOnTop: false,
        incognito: false,
        type: 'normal',
        id: windowId,
        left: 0,
        sessionId: '11bf5b37-e0b8-42e0-8dcf-dc8c4aefc000',
      };

      chrome.windows.getLastFocused.mockImplementation(
        (
          _getInfo: chrome.windows.GetInfo,
          callback: (window: chrome.windows.Window) => void
        ) => callback?.(returnedWindow)
      );
      chrome.tabs.query.mockImplementation(
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        (
          _queryInfo: chrome.tabs.QueryInfo,
          callback: (tabs: chrome.tabs.Tab[]) => void
        ) => {
          callback?.(returnedTabs);
        }
      );

      // Execute
      const result = await new WindowsService().getActiveTabs();

      // Assert
      const expectedResult = returnedTabs;
      expect(result).toBe(expectedResult);
      expect(chrome.tabs.query).toHaveBeenCalled();
    });

    it('When there is no active tabs, getActiveTabs return empty', async () => {
      // Prepare
      const returnedWindow: chrome.windows.Window = {
        top: 0,
        height: 400,
        width: 600,
        state: 'normal',
        focused: true,
        alwaysOnTop: false,
        incognito: false,
        type: 'normal',
        id: 12,
        left: 0,
        sessionId: '11bf5b37-e0b8-42e0-8dcf-dc8c4aefc000',
      };

      chrome.windows.getLastFocused.mockImplementation(
        (
          _getInfo: chrome.windows.GetInfo,
          callback: (window: chrome.windows.Window) => void
        ) => callback?.(returnedWindow)
      );
      chrome.tabs.query.mockImplementation(
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        (
          _queryInfo: chrome.tabs.QueryInfo,
          callback: (tabs: chrome.tabs.Tab[]) => void
        ) => {
          callback?.([]);
        }
      );

      // Execute
      const result = await new WindowsService().getActiveTabs();

      // Assert
      expect(result).toHaveLength(0);
      expect(chrome.tabs.query).toHaveBeenCalled();
    });
  });
});
