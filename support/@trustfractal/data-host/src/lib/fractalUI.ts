// import App from '@components/App';
import App from '@components/App';
import ReactDOM from 'react-dom';


export class FractalUI {
  //TODO(melatron): Add substrateAddress that would be used for sending funds
  render(element: HTMLElement | null): void {
    ReactDOM.render(App(), element);
  }
}
