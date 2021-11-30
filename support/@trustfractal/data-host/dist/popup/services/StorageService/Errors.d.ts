/// <reference types="chrome" />
declare enum ErrorCode {
    ERROR_HAS_ITEM = 1000,
    ERROR_GET_ITEM = 1001,
    ERROR_SET_ITEM = 1002,
    ERROR_REMOVE_ITEM = 1003,
    ERROR_CLEAR = 1003
}
declare class StorageServiceError extends Error {
    errorCode: ErrorCode;
    errorChrome: chrome.runtime.LastError;
    constructor(errorCode: ErrorCode, errorChrome: chrome.runtime.LastError, message: string);
}
export declare const ERROR_HAS_ITEM: (errorChrome: chrome.runtime.LastError, key: string) => StorageServiceError;
export declare const ERROR_GET_ITEM: (errorChrome: chrome.runtime.LastError, key: string) => StorageServiceError;
export declare const ERROR_SET_ITEM: (errorChrome: chrome.runtime.LastError, key: string, value: string) => StorageServiceError;
export declare const ERROR_REMOVE_ITEM: (errorChrome: chrome.runtime.LastError, key: string) => StorageServiceError;
export declare const ERROR_CLEAR: (errorChrome: chrome.runtime.LastError) => StorageServiceError;
export {};
