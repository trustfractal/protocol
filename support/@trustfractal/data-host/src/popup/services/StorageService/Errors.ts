enum ErrorCode {
  ERROR_HAS_ITEM = 1000,
  ERROR_GET_ITEM = 1001,
  ERROR_SET_ITEM = 1002,
  ERROR_REMOVE_ITEM = 1003,
  ERROR_CLEAR = 1003,
}

class StorageServiceError extends Error {
  public errorCode: ErrorCode;
  public errorChrome: chrome.runtime.LastError;

  public constructor(
    errorCode: ErrorCode,
    errorChrome: chrome.runtime.LastError,
    message: string
  ) {
    super(message);
    this.errorChrome = errorChrome;
    this.errorCode = errorCode;
  }
}

export const ERROR_HAS_ITEM = (
  errorChrome: chrome.runtime.LastError,
  key: string
): StorageServiceError => {
  return new StorageServiceError(
    ErrorCode.ERROR_HAS_ITEM,
    errorChrome,
    `StorageService: could not check if key '${key}' is set`
  );
};

export const ERROR_GET_ITEM = (
  errorChrome: chrome.runtime.LastError,
  key: string
): StorageServiceError => {
  return new StorageServiceError(
    ErrorCode.ERROR_GET_ITEM,
    errorChrome,
    `StorageService: could not get key '${key}'`
  );
};

export const ERROR_SET_ITEM = (
  errorChrome: chrome.runtime.LastError,
  key: string,
  value: string
): StorageServiceError => {
  return new StorageServiceError(
    ErrorCode.ERROR_SET_ITEM,
    errorChrome,
    `StorageService: could not set value '${value}' for key '${key}'`
  );
};

export const ERROR_REMOVE_ITEM = (
  errorChrome: chrome.runtime.LastError,
  key: string
): StorageServiceError => {
  return new StorageServiceError(
    ErrorCode.ERROR_REMOVE_ITEM,
    errorChrome,
    `StorageService: could not remove key '${key}'`
  );
};

export const ERROR_CLEAR: (
  errorChrome: chrome.runtime.LastError
) => StorageServiceError = (
  errorChrome: chrome.runtime.LastError
): StorageServiceError => {
  return new StorageServiceError(
    ErrorCode.ERROR_CLEAR,
    errorChrome,
    `StorageService: could not clear`
  );
};
