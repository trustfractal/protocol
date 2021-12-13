
import { Store } from 'webext-redux';

import { REDUX_PORT_NAME } from '../../redux/port';

import { FractalUI } from '@trustfractal/data-host';
import './index.css';

const store = new Store({ portName: REDUX_PORT_NAME });

store.ready().then(async () => {
    let ui = new FractalUI();
    ui.render(document.getElementById('app-container'));
});

if (module.hot) module.hot.accept();