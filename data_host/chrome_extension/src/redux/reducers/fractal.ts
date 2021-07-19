import { PayloadAction } from '@reduxjs/toolkit';

import type { FractalData } from '@redux/state';
import { ADD_FRACTAL_ID } from '@redux/actionTypes';

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
