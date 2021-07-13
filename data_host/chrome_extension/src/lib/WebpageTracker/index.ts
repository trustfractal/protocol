import type { Location } from './types';
export * from './types';

export type WebpageTracker = Record<string, Record<string, number>>;

export const build = (): WebpageTracker => ({});

export const add = (
  counter: WebpageTracker,
  { pathname, hostname }: Location
): WebpageTracker => {
  const webpage = counter[hostname] || {};
  const count = webpage[pathname] || 0;

  return {
    ...counter,
    [hostname]: {
      ...webpage,
      [pathname]: count + 1,
    },
  };
};

const exports = { add, build };

export default exports;
