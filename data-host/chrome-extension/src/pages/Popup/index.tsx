
// import { Store } from 'webext-redux';

import { REDUX_PORT_NAME } from '../../redux/port';
import './index.css';
// import '@trustfractal/6fab01f2c0d529f29293.module.wasm';
let { FractalUI } = await import('@trustfractal/data-host');

// const store = new Store({ portName: REDUX_PORT_NAME });

// store.ready().then(async () => {
    let ui = new FractalUI();
    ui.render(document.getElementById('app-container'));
// });

if (module.hot) module.hot.accept();
