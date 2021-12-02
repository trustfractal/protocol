import { State } from './state';

export const getWebpages = (state: State) => {
  console.log('GET WEBPAGES STATE ', state);
  return state.webpages;
};

export const getFractalData = (state: State) => {
  console.log('GET FRACTAL STATE ', state);
  return state.fractal;
};

// export const getFractalData = ({ fractal }: State) => fractal;
