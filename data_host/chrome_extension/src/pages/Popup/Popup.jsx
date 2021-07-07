import React from 'react';
import { useSelector } from 'react-redux';

import { getWebpages } from '../../redux/selectors';

import './Popup.css';

const renderPath = (pathname, counter, key) => (
  <li key={key}>
    {pathname} rendered {counter} times
  </li>
);

const renderWebpage = (hostname, paths, key) => {
  console.log('hostname: ', hostname);
  console.log('paths: ', paths);

  return (
    <div key={key}>
      <h2>{hostname}</h2>
      <ul>
        {Object.entries(paths).map(([pathname, counter], i) =>
          renderPath(pathname, counter, i)
        )}
      </ul>
    </div>
  );
};

const Popup = () => {
  const { webpages } = useSelector(getWebpages);

  console.log('Known webpages: ', webpages);

  return (
    <div className="App">
      <header className="App-header">
        <h1>Visited websites</h1>
        {Object.entries(webpages).map(([hostname, paths], i) =>
          renderWebpage(hostname, paths, i)
        )}
      </header>
    </div>
  );
};

export default Popup;
