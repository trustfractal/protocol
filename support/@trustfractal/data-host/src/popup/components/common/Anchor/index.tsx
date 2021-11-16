import React from "react";

import styled from "styled-components";

const Root = styled.a`
  color: var(--c-orange);
  background: var(--c-transparent);
  text-decoration: underline;
  font-weight: normal;
`;

type AnchorProps = {
  link: string;
};

function Anchor(props: AnchorProps & React.HTMLAttributes<HTMLAnchorElement>) {
  const { children, link, ...otherProps } = props;

  return (
    <Root href={link} {...otherProps}>
      {children}
    </Root>
  );
}

export default Anchor;
