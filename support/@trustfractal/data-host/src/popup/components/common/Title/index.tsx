import React from "react";

import styled from "styled-components";

const Root = styled.h2`
  font-size: var(--s-20);
  line-height: var(--s-23);
  font-weight: bold;
  margin-bottom: var(--s-12);
  text-align: center;
`;

function Title(props: React.HTMLProps<HTMLHeadingElement>) {
  const { children } = props;

  return <Root>{children}</Root>;
}

export default Title;
