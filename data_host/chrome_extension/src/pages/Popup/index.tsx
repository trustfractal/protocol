import React from 'react';
import { render } from 'react-dom';
import { Provider } from 'react-redux';
import { Store } from 'webext-redux';

import { REDUX_PORT_NAME } from '../../redux/port';

import {FractalUI, Background} from '@trustfractal/data-host';
import Popup from './Popup';
import './index.css';

const store = new Store({ portName: REDUX_PORT_NAME });

store.ready().then(() => {
    let ui = new FractalUI();
    //TODO: `ui.render` wouldn't return a promise. Change it to `ui.render(document.getElementById('app-container'));`
    // when the new version of @trustfractal/data-host is up.
    ui.render(document.getElementById('app-container')).then(() => {
       console.log('FractalUI render promise called.')
    });
});

if (module.hot) module.hot.accept();
