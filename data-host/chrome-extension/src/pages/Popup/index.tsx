
import { FractalUI } from '@trustfractal/data-host';
import './index.css';

new FractalUI().render(document.getElementById('app-container'));

if (module.hot) module.hot.accept();
