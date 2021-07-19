import React from 'react';
import { render } from 'react-dom';
import { Provider } from 'react-redux';
import { Store } from 'webext-redux';

import { REDUX_PORT_NAME } from '@redux/port';

import Popup from './Popup';

import '../reset.css';
import './index.css';

const store = new Store({ portName: REDUX_PORT_NAME });

store.ready().then(() => {
  render(
    <Provider store={store}>
      <Popup />
    </Provider>,

    window.document.querySelector('#app-container')
  );
});

if (module.hot) module.hot.accept();
