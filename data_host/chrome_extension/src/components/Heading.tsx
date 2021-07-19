import styled from 'styled-components';

import { Sizes } from '@components/Theme';

type Level = '1' | '2' | '3' | '4' | '5';

interface Props {
  level?: Level;
  children: React.ReactNode;
}

const LevelSizeMapping = {
  1: Sizes.xl,
  2: Sizes.l,
  3: Sizes.m,
  4: Sizes.s,
  5: Sizes.xs,
};

const sizeFromProps = ({ level }: Props) =>
  LevelSizeMapping[level || '1'] || Sizes.xl;

const Heading = styled.span`
  font-weight: bold;
  font-size: ${sizeFromProps};
`;

export default Heading;
