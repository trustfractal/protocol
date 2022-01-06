
import { FractalUI } from '@trustfractal/data-host';
import './index.css';

let ui = new FractalUI();
ui.render(document.getElementById('app-container'));


if (module.hot) module.hot.accept();
