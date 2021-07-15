import { PayloadAction } from '@reduxjs/toolkit';

import type { State } from '../state';
import { initialState } from '../state';
import { ADD_FRACTAL_ID } from '../actionTypes';

const reducers = (
  state: State = initialState,
  action: PayloadAction<object>
) => {
  const { fractal } = state;

  switch (action.type) {
    case ADD_FRACTAL_ID:
      return {
        ...state,
        fractal: { ...fractal, id: action.payload },
      };

    default:
      return state;
  }
};

export default reducers;
