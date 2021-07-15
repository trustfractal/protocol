import { PayloadAction } from '@reduxjs/toolkit';

import type { FractalData } from '../state';
import { ADD_FRACTAL_ID } from '../actionTypes';

const initialState = { id: null };

const reducers = (
  fractal: FractalData = initialState,
  action: PayloadAction<object>
) => {
  switch (action.type) {
    case ADD_FRACTAL_ID:
      return { ...fractal, id: action.payload };

    default:
      return fractal;
  }
};

export default reducers;
