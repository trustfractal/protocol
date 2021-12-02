// import App from '@components/App';
import App from '@components/App';
import { changeEnviroment } from '@popup/Environment';
import { Environment } from '@popup/types/Environment';
import ReactDOM from 'react-dom';

export class FractalUI {
  init(settings: Environment) {
    changeEnviroment(settings);
  }
  //TODO(melatron): Add substrateAddress that would be used for sending funds
  render(element: HTMLElement | null): void {
    ReactDOM.render(App(), element);
  }
}
