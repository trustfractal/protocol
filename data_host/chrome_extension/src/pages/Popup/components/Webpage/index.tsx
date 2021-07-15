import React from 'react';

import Heading from '../../../../components/Heading';
import Spacing from '../../../../components/Spacing';

interface Props {
  hostname: string;
  paths: Record<string, number>;
}

const renderPath = (pathname: string, counter: number, key: number) => (
  <p key={key}>
    {pathname} rendered {counter} times
  </p>
);

const Webpage = ({ hostname, paths }: Props) => (
  <div>
    <Heading level="2">{hostname}</Heading>

    <Spacing size="xs" />

    {Object.entries(paths).map(([pathname, counter], i) =>
      renderPath(pathname, counter, i)
    )}
  </div>
);

export default Webpage;
