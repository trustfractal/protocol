import React from 'react';
import { useSelector } from 'react-redux';
import styled from 'styled-components';

import { getWebpages, getFractalData } from '../../redux/selectors';
import { WebpageTracker, FractalData } from '../../redux/state';

import Form from './components/Form';
import Webpage from './components/Webpage';

import Heading from '../../components/Heading';
import Error from '../../components/Error';
import Section from '../../components/Section';
import Spacing from '../../components/Spacing';
import Theme from '../../components/Theme';

const Container = styled.div`
  position: absolute;
  top: 0px;
  bottom: 0px;
  left: 0px;
  right: 0px;
  height: 100%;

  display: flex;
  flex-direction: column;
  padding: ${Theme.Sizes.xl};

  background-color: ${Theme.Pallette.EerieBlack};
  color: ${Theme.Pallette.Platinum};

  font-size: ${Theme.Sizes.m};
`;

const renderFractal = (fractal: FractalData) => {
  const { id } = fractal;

  return id ? renderFractalData(fractal) : renderIDForm();
};

const renderIDForm = () => (
  <>
    <Error>Your Fractal ID hasn't been set.</Error>
    <Spacing size="xl" />
    <Form />
  </>
);

const renderFractalData = ({ id }: FractalData) => (
  <>
    <Heading>Fractal Data</Heading>

    <Spacing size="m" />

    <p>
      <strong>ID:</strong> {id}
    </p>
  </>
);

const renderVisits = (webpages: WebpageTracker) => (
  <>
    <Heading>Visited websites</Heading>
    <Spacing size="l" />
    {renderWebpages(webpages)}
  </>
);

const renderWebpages = (webpages: WebpageTracker) => {
  const entries = Object.entries(webpages);

  if (entries.length === 0)
    return <span>You haven't visited anything yet.</span>;

  return entries.map(([hostname, paths], i) => (
    <div key={i}>
      <Webpage hostname={hostname} paths={paths} />
      <Spacing size="l" />
    </div>
  ));
};

const Popup = () => {
  const webpages = useSelector(getWebpages);
  const fractal = useSelector(getFractalData);

  return (
    <Container>
      <Section>{renderFractal(fractal)}</Section>

      <Spacing size="xl" />

      <Section>{renderVisits(webpages)}</Section>
    </Container>
  );
};

export default Popup;
