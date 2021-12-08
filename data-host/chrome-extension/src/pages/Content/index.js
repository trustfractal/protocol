import { Store } from 'webext-redux';

import { REDUX_PORT_NAME } from '../../redux/port';
import { addWebpage } from '../../redux/actions';

(() => {
  const store = new Store({ portName: REDUX_PORT_NAME });

  store.ready().then(() => {
    store.dispatch(addWebpage(window.location));
  });
})();
