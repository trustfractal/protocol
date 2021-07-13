import { ADD_WEBPAGE } from './actionTypes';

import { Location } from '../lib/WebpageTracker';

export const addWebpage = (location: Location) => ({
  type: ADD_WEBPAGE,
  payload: location,
});
