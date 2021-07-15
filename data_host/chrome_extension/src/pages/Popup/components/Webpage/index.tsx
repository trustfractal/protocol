import React from 'react';

interface Props {
  hostname: string;
  paths: Record<string, number>;
}

const renderPath = (pathname: string, counter: number, key: number) => (
  <li key={key}>
    {pathname} rendered {counter} times
  </li>
);

const Webpage = ({ hostname, paths }: Props) => (
  <div>
    <h3>{hostname}</h3>
    <ul>
      {Object.entries(paths).map(([pathname, counter], i) =>
        renderPath(pathname, counter, i)
      )}
    </ul>
  </div>
);

export default Webpage;
