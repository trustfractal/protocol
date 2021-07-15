import { ADD_FRACTAL_ID, ADD_WEBPAGE } from './actionTypes';

import { Location } from '../lib/WebpageTracker';

export const addWebpage = (location: Location) => ({
  type: ADD_WEBPAGE,
  payload: location,
});

export const addFractalID = (id: string) => ({
  type: ADD_FRACTAL_ID,
  payload: id,
});
