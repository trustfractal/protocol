import { PayloadAction } from '@reduxjs/toolkit';

import type { WebpageTracker } from '../state';

import Tracker, { Location } from '../../lib/WebpageTracker';
import { ADD_WEBPAGE } from '../actionTypes';

const initialState = {};

const reducers = (
  webpages: WebpageTracker = initialState,
  action: PayloadAction<Location>
) => {
  switch (action.type) {
    case ADD_WEBPAGE:
      return Tracker.add(webpages, action.payload);

    default:
      return webpages;
  }
};

export default reducers;
