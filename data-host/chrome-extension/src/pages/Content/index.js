import { InjectionScript } from '@trustfractal/data-host';
(() =>{
  let injectionScript = new InjectionScript();
  console.log('new injection')
  injectionScript.initialize('').then(() => {
    injectionScript.sendCurrentPageView();
    console.log('send view')
  });
  console.log('init injection')


})();
