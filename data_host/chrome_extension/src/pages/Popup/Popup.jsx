import React from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { Form, Field } from 'react-final-form';

import { getWebpages, getFractalData } from '../../redux/selectors';
import { addFractalID } from '../../redux/actions';

import './Popup.css';

const renderPath = (pathname, counter, key) => (
  <li key={key}>
    {pathname} rendered {counter} times
  </li>
);

const renderWebpage = (hostname, paths, key) => {
  return (
    <div key={key}>
      <h3>{hostname}</h3>
      <ul>
        {Object.entries(paths).map(([pathname, counter], i) =>
          renderPath(pathname, counter, i)
        )}
      </ul>
    </div>
  );
};

const renderWebpages = (webpages) =>
  Object.entries(webpages).map(([hostname, paths], i) =>
    renderWebpage(hostname, paths, i)
  );

const renderFractalData = ({ id }) => (
  <ul>
    <li key="fractal-id">
      <strong>ID:</strong> {id || 'No ID set'}
    </li>
  </ul>
);

const onSubmit = async (dispatch, { fractalID }, form) => {
  await dispatch(addFractalID(fractalID));
  form.reset();
};

const validate = (data) => {
  switch (true) {
    case Object.keys(data).length === 0:
      return {};
    case !data.fractalID.startsWith('0x'):
      return {
        fractalID: 'Expected hex-encoded byte sequence beginning with "0x"',
      };
    default:
      return {};
  }
};

const renderIDForm = ({ handleSubmit }) => (
  <form onSubmit={handleSubmit}>
    <Field name="fractalID">
      {({ input, meta }) => (
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'flex-start',
          }}
        >
          <label style={{ fontWeight: 'bold' }}>Fractal ID</label>
          <input {...input} type="text" placeholder="Fractal ID" />
          {meta.error && meta.touched && (
            <span style={{ color: 'red' }}>{meta.error}</span>
          )}
        </div>
      )}
    </Field>
    <button type="submit">Set ID</button>
  </form>
);

const renderActions = (dispatch) => (
  <div style={{ display: 'flex', flexDirection: 'column' }}>
    <h3>Update ID</h3>

    <Form
      onSubmit={async (data, form) => await onSubmit(dispatch, data, form)}
      validate={validate}
      render={renderIDForm}
    />
  </div>
);

const Popup = () => {
  const { webpages } = useSelector(getWebpages);
  const { fractal } = useSelector(getFractalData);
  const dispatch = useDispatch();

  return (
    <div className="App">
      <header className="App-header">
        <h2>Fractal Data</h2>
        {renderFractalData(fractal)}

        <h2>Visited websites</h2>
        {renderWebpages(webpages)}

        <h2>Actions</h2>
        {renderActions(dispatch)}
      </header>
    </div>
  );
};

export default Popup;
