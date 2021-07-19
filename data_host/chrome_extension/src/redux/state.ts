import { WebpageTracker as WebpageTrackerType } from '@lib/WebpageTracker';

export type WebpageTracker = WebpageTrackerType;

export type FractalData = {
  id: string | null;
};

export type State = {
  webpages: WebpageTracker;
  fractal: FractalData;
};
