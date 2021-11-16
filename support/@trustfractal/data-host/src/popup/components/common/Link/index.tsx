import React from "react";

import Text, {
  TextProps,
  TextSizes,
  TextHeights,
  TextWeights,
} from "@popup/components/common/Text";

import styled from "styled-components";

const Root = styled.span`
  cursor: pointer;
  color: var(--c-orange);
  text-decoration: underline;
`;

type LinkProps = {
  onClick: () => void;
};

Link.defaultProps = {
  size: TextSizes.MEDIUM,
  height: TextHeights.MEDIUM,
  weight: TextWeights.NORMAL,
  span: false,
};

function Link(
  props: LinkProps & TextProps & React.HTMLAttributes<HTMLDivElement>,
) {
  const { children, onClick, ...otherProps } = props;

  return (
    <Root onClick={onClick}>
      <Text {...otherProps}>{children}</Text>
    </Root>
  );
}

export default Link;
