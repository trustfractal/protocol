import styled from 'styled-components';

import { Sizes } from './Theme';
import type { Size } from './Theme';

interface Props {
  size: Size;
}

const sizeFromProps = ({ size }: Props) => Sizes[size];

const Spacing = styled.div`
  margin: 0 0 ${sizeFromProps} 0;
  padding: 0;
`;

export default Spacing;
