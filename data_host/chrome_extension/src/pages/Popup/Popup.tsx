import React from 'react';
import { useSelector } from 'react-redux';

import { getWebpages, getFractalData } from '../../redux/selectors';
import { WebpageTracker, FractalData } from '../../redux/state';

import Form from './components/Form';
import Webpage from './components/Webpage';

import './Popup.css';

const renderWebpages = (webpages: WebpageTracker) =>
  Object.entries(webpages).map(([hostname, paths], i) => (
    <div key={i}>
      <Webpage hostname={hostname} paths={paths} />
    </div>
  ));

const renderFractalData = ({ id }: FractalData) => (
  <ul>
    <li key="fractal-id">
      <strong>ID:</strong> {id || 'No ID set'}
    </li>
  </ul>
);

const renderActions = () => (
  <div style={{ display: 'flex', flexDirection: 'column' }}>
    <h3>Update ID</h3>

    <Form />
  </div>
);

const Popup = () => {
  const webpages = useSelector(getWebpages);
  const fractal = useSelector(getFractalData);

  return (
    <div className="App">
      <header className="App-header">
        <h2>Fractal Data</h2>
        {renderFractalData(fractal)}

        <h2>Visited websites</h2>
        {renderWebpages(webpages)}

        <h2>Actions</h2>
        {renderActions()}
      </header>
    </div>
  );
};

export default Popup;
