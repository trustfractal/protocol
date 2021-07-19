import { createStore } from 'redux';
import { wrapStore } from 'webext-redux';

import reducers from '@redux/reducers';
import { REDUX_PORT_NAME } from '@redux/port';

const store = createStore(reducers);
wrapStore(store, { portName: REDUX_PORT_NAME });

(() => {
  console.log('Background ready');
})();
