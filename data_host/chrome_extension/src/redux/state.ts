import { WebpageTracker } from '../lib/WebpageTracker';

export type FractalData = {
  id: string | null;
};

export type State = {
  webpages: WebpageTracker;
  fractal: FractalData;
};

export const initialState: State = {
  webpages: {},
  fractal: { id: null },
};
