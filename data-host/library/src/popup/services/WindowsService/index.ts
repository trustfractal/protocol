import { environment } from '@popup/Environment';

import {
  ERROR_CLOSE_WINDOW,
  ERROR_CREATE_TAB,
  ERROR_CREATE_WINDOW,
  ERROR_FOCUS_WINDOW,
  ERROR_GET_ALL_WINDOWS,
  ERROR_GET_CURRENT_WINDOW,
  ERROR_GET_LAST_FOCUSED_WINDOW,
  ERROR_GET_TAB,
  ERROR_GET_WINDOW,
  ERROR_QUERY_TABS,
  ERROR_UPDATE_TAB,
  ERROR_UPDATE_WINDOW,
  ERROR_UPDATE_WINDOW_POSITION,
  ERROR_UPDATE_WINDOW_SIZE,
} from './Errors';

export enum PopupSizes {
  SMALL = 'small',
  MEDIUM = 'medium',
  LARGE = 'large',
}

export const PopupSizesValues: Record<
  PopupSizes,
  { width: number; height: number }
> = {
  [PopupSizes.SMALL]: {
    width: 400,
    height: 460,
  },
  [PopupSizes.MEDIUM]: {
    width: 400,
    height: 600,
  },
  [PopupSizes.LARGE]: {
    width: 400,
    height: 740,
  },
};

export class WindowsService {
  private popupId?: number;

  createWindow(
    config: chrome.windows.CreateData = {}
  ): Promise<chrome.windows.Window | undefined> {
    return new Promise((resolve, reject) => {
      chrome.windows.create(config, (window) => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(ERROR_CREATE_WINDOW(chrome.runtime.lastError));
        }

        resolve(window);
      });
    });
  }

  getCurrentWindow(
    config: chrome.windows.GetInfo = {}
  ): Promise<chrome.windows.Window> {
    return new Promise((resolve, reject) => {
      chrome.windows.getCurrent(config, (window) => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(ERROR_GET_CURRENT_WINDOW(chrome.runtime.lastError));
        }

        resolve(window);
      });
    });
  }

  getLastFocusedWindow(): Promise<chrome.windows.Window> {
    return new Promise((resolve, reject) => {
      chrome.windows.getLastFocused((window) => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(ERROR_GET_LAST_FOCUSED_WINDOW(chrome.runtime.lastError));
        }

        resolve(window);
      });
    });
  }

  getAllWindows(
    config: chrome.windows.GetInfo = {}
  ): Promise<Array<chrome.windows.Window>> {
    return new Promise((resolve, reject) => {
      chrome.windows.getAll(config, (windows) => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(ERROR_GET_ALL_WINDOWS(chrome.runtime.lastError));
        }

        resolve(windows);
      });
    });
  }

  focusWindow(windowId: number): Promise<void> {
    return new Promise((resolve, reject) => {
      chrome.windows.update(windowId, { focused: true }, () => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(ERROR_FOCUS_WINDOW(chrome.runtime.lastError));
        }

        resolve();
      });
    });
  }

  async updateWindowPosition(
    windowId: number,
    left: number,
    top: number
  ): Promise<void> {
    try {
      await this.updateWindow(windowId, { left, top });
    } catch (error) {
      if (chrome.runtime.lastError !== undefined) {
        throw ERROR_UPDATE_WINDOW_POSITION(chrome.runtime.lastError);
      }

      throw error;
    }
  }

  async updateWindowSize(
    windowId: number,
    width: number,
    height: number
  ): Promise<void> {
    try {
      await this.updateWindow(windowId, { width, height });
    } catch (error) {
      if (chrome.runtime.lastError !== undefined) {
        throw ERROR_UPDATE_WINDOW_SIZE(chrome.runtime.lastError);
      }

      throw error;
    }
  }

  closeWindow(windowId: number): Promise<void> {
    return new Promise((resolve, reject) => {
      chrome.windows.onRemoved.addListener((closedWindowId) => {
        if (windowId === closedWindowId) resolve();
      });

      chrome.windows.remove(windowId, () => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(ERROR_CLOSE_WINDOW(chrome.runtime.lastError));
        }
      });
    });
  }

  async closeCurrentWindow(): Promise<chrome.windows.Window> {
    const window = await this.getCurrentWindow();

    await this.closeWindow(window.id);

    return window;
  }

  async closeCurrentPopup(): Promise<chrome.windows.Window> {
    const window = await this.getCurrentWindow({ windowTypes: ['popup'] });

    if (window) await this.closeWindow(window.id);

    return window;
  }

  async closeAllWindows(): Promise<Array<chrome.windows.Window>> {
    const windows = await this.getAllWindows();

    for (let index = 0; index < windows.length; index++) {
      const { id } = windows[index];

      await this.closeWindow(id);
    }

    return windows;
  }

  async isNativePopupOpen() {
    return chrome.extension.getViews({ type: 'popup' }).length > 0;
  }

  async createPopup(
    size: PopupSizes = PopupSizes.SMALL
  ): Promise<chrome.windows.Window | undefined> {
    const popup = await this.getPopup();
    const popupSize = PopupSizesValues[size];

    if (popup) {
      // bring focus to existing chrome popup
      await this.focusWindow(popup.id);
      return;
    }

    // check native popup is open
    const isOpen = await this.isNativePopupOpen();
    if (isOpen) {
      return;
    }

    let left = 0;
    let top = 0;

    const lastFocused = await this.getLastFocusedWindow();
    if (lastFocused.top === undefined) {
      const { screenY } = window;

      top = Math.max(screenY, 0);
    } else {
      top = lastFocused.top;
    }

    if (lastFocused.left === undefined || lastFocused.width === undefined) {
      const { screenX, outerWidth } = window;
      left = Math.max(screenX + (outerWidth - popupSize.width), 0);
    } else {
      left = lastFocused.left + (lastFocused.width - popupSize.width);
    }

    // create new notification popup
    const popupWindow = await this.createWindow({
      url: 'popup.html',
      type: 'popup',
      width: popupSize.width,
      height: popupSize.height,
      left,
      top,
    });

    if (popupWindow !== undefined) {
      if (popupWindow.left !== left && popupWindow.state !== 'fullscreen') {
        await this.updateWindowPosition(popupWindow.id, left, top);
      }

      this.popupId = popupWindow.id;
    }

    return popupWindow;
  }

  public async closePopup() {
    const window = await this.getPopup();

    if (window !== undefined) {
      await this.closeWindow(window.id);
    }
  }

  public async getPopup() {
    const windows = await this.getAllWindows();

    if (windows === undefined || windows.length === 0) {
      return;
    }

    return windows.find(
      (win) => win && win.type === 'popup' && win.id === this.popupId
    );
  }

  async getAllPopups(): Promise<Array<chrome.windows.Window>> {
    return this.getAllWindows({
      windowTypes: ['popup'],
    });
  }

  async closeAllPopups(): Promise<Array<chrome.windows.Window>> {
    const popups = await this.getAllPopups();

    for (let index = 0; index < popups.length; index++) {
      const window = popups[index];

      await this.closeWindow(window.id);
    }

    return popups;
  }

  getWindow(
    windowId: number,
    config: chrome.windows.GetInfo = {}
  ): Promise<chrome.windows.Window> {
    return new Promise((resolve, reject) => {
      chrome.windows.get(windowId, config, (window) => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(ERROR_GET_WINDOW(chrome.runtime.lastError, windowId));
        }

        resolve(window);
      });
    });
  }

  updateWindow(
    windowId: number,
    config: chrome.windows.UpdateInfo
  ): Promise<chrome.windows.Window | undefined> {
    return new Promise((resolve, reject) => {
      chrome.windows.update(windowId, config, (updateWindow) => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(ERROR_UPDATE_WINDOW(chrome.runtime.lastError, windowId));
        }

        resolve(updateWindow);
      });
    });
  }

  getTab(tabId: number): Promise<chrome.tabs.Tab> {
    return new Promise((resolve, reject) => {
      chrome.tabs.get(tabId, (tab) => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(ERROR_GET_TAB(chrome.runtime.lastError, tabId));
        }

        resolve(tab);
      });
    });
  }

  createTab(
    properties: chrome.tabs.CreateProperties
  ): Promise<chrome.tabs.Tab> {
    return new Promise((resolve, reject) => {
      chrome.tabs.create(properties, (tab) => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(ERROR_CREATE_TAB(chrome.runtime.lastError));
        }

        resolve(tab);
      });
    });
  }

  updateTab(
    tabId: number,
    config: chrome.tabs.UpdateProperties
  ): Promise<chrome.tabs.Tab | undefined> {
    return new Promise((resolve, reject) => {
      chrome.tabs.update(tabId, config, (updatedtab) => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(ERROR_UPDATE_TAB(chrome.runtime.lastError, tabId));
        }

        resolve(updatedtab);
      });
    });
  }

  queryTabs(queryInfo: chrome.tabs.QueryInfo = {}): Promise<chrome.tabs.Tab[]> {
    return new Promise((resolve, reject) => {
      chrome.tabs.query(queryInfo, (tabs) => {
        if (chrome.runtime.lastError !== undefined) {
          console.error(chrome.runtime.lastError);
          reject(ERROR_QUERY_TABS(chrome.runtime.lastError));
        }

        resolve(tabs);
      });
    });
  }

  redirectTab(id: number, url: string) {
    return this.updateTab(id, { url });
  }

  async getActiveTabs(): Promise<chrome.tabs.Tab[]> {
    return new Promise<chrome.tabs.Tab[]>((resolve) => {
      // get last normal window focused
      chrome.windows.getLastFocused(
        {
          windowTypes: ['normal'],
        },
        async (lastWindowFocused) => {
          // get window active tab
          const tabs = await this.queryTabs({
            windowId: lastWindowFocused.id,
            active: true,
          });

          resolve(tabs);
        }
      );
    });
  }

  async getFractalTabs(): Promise<chrome.tabs.Tab[]> {
    const { hostname } = new URL(environment.FRACTAL_WEBSITE_URL);

    const senderHostname = hostname.startsWith('www.')
      ? hostname.substr(4)
      : hostname;

    // get fractal tab
    const tabs = await this.queryTabs({
      url: `*://*.${senderHostname}/*`,
    });

    return tabs;
  }

  async openTab(url: string) {
    const activeTabs = await this.getActiveTabs();

    if (activeTabs.length === 0) {
      return this.createTab({ url });
    }

    const [activeTab] = activeTabs;

    if (activeTab.id === undefined) {
      return this.createTab({ url });
    }

    if (activeTab.url?.includes('fractal.id')) {
      return this.redirectTab(activeTab.id, url);
    }

    return this.createTab({ url });
  }
}
