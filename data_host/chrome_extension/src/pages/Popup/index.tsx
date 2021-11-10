import React from 'react';
import { render } from 'react-dom';
import { Provider } from 'react-redux';
import { Store } from 'webext-redux';

import { REDUX_PORT_NAME } from '../../redux/port';

import {FractalUI} from '@trustfractal/data-host';
import Popup from './Popup';
import './index.css';

const store = new Store({ portName: REDUX_PORT_NAME });

store.ready().then(() => {
    let ui = new FractalUI();
    ui.render(document.getElementById('#app-container')).then(() => {
       console.log('FractalUI setup promise called.')
    });

//   render(
//     <Provider store={store}>
//       <Popup />
//     </Provider>,
//     window.document.querySelector('#app-container')
//   );
});

if (module.hot) module.hot.accept();
