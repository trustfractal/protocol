// import { Store } from 'webext-redux';

// import { REDUX_PORT_NAME } from '../../redux/port';
// import { addWebpage } from '../../redux/actions';
import { InjectionScript } from '@trustfractal/data-host';
(() => {
//   const store = new Store({ portName: REDUX_PORT_NAME });
  const injectionScript = new InjectionScript();
  injectionScript.sendCurrentPageView();
//   store.ready().then(() => {
//     store.dispatch(addWebpage(window.location));
//   });
})();
