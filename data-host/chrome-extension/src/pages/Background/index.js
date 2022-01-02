import { createStore } from 'redux';
import { wrapStore, Store } from 'webext-redux';

import reducers from '../../redux/reducers';
import { REDUX_PORT_NAME } from '../../redux/port';
import { Background } from '@trustfractal/data-host';
const store = createStore(reducers);
wrapStore(store, { portName: REDUX_PORT_NAME });

(() => {

  console.log(store.getState())
    const background = new Background();
    store.subscribe(() => {
        // store.getState().webpages
        let lastPage = Object.keys(store.getState().webpages).at(-1) || '';
        console.log(lastPage);
        background.addWebpage(lastPage);
    });
})();
