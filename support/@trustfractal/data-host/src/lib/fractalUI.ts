// import App from '@components/App';
import ReactDOM from 'react-dom';

import App from '../popup/components/App';

export class FractalUI {
  // substrateAddress is going to be used to send funds
  async render(element: HTMLElement | null): Promise<void> {
    ReactDOM.render(App(), element);
  }
}
