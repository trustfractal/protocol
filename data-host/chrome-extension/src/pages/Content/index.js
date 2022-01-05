// import { InjectionScript } from '@trustfractal/data-host';
(() => {
//   const injectionScript = new InjectionScript();
//   injectionScript.sendCurrentPageView();
    console.log('------------------------')
    chrome.runtime.sendMessage({ type: 'pageView', content: window.location });
    console.log('------------------------')
})();
