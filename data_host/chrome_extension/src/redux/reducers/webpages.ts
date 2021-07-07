import { PayloadAction } from '@reduxjs/toolkit';

import WebpageTracker, { Location } from '../../lib/WebpageTracker';
import type { State } from '../types';
import { ADD_WEBPAGE } from '../actionTypes';

const initialState = {
  webpages: {},
};

const reducers = (
  state: State = initialState,
  action: PayloadAction<Location>
) => {
  switch (action.type) {
    case ADD_WEBPAGE:
      return {
        ...state,
        webpages: WebpageTracker.add(state.webpages, action.payload),
      };

    default:
      return state;
  }
};

export default reducers;
