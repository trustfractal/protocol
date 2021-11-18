// import App from '@components/App';
import ReactDOM from 'react-dom';

import App from '../popup/components/App';

export class FractalUI {
  //TODO(melatron): Add substrateAddress that would be used for sending funds
  render(element: HTMLElement | null): void {
    ReactDOM.render(App(), element);
  }
}
