import React from "react";
import styled, { css } from "styled-components";

const Root = styled.div<{
  paddingTop: string;
  paddingBottom: string;
  paddingLeft: string;
  paddingRight: string;
}>`
  position: relative;

  ${(props) => css`
    padding-top: ${props.paddingTop};
    padding-bottom: ${props.paddingBottom};
    padding-left: ${props.paddingLeft};
    padding-right: ${props.paddingRight};
  `}
`;

type TopComponentProps = {
  paddingTop: string;
  paddingBottom: string;
  paddingLeft: string;
  paddingRight: string;
};

TopComponent.defaultProps = {
  paddingTop: "var(--s-24)",
  paddingBottom: "var(--s-24)",
  paddingLeft: "var(--s-24)",
  paddingRight: "var(--s-24)",
};

function TopComponent(
  props: TopComponentProps & React.HTMLProps<HTMLDivElement>,
) {
  const { children, paddingTop, paddingBottom, paddingLeft, paddingRight } =
    props;

  return (
    <Root
      paddingTop={paddingTop}
      paddingBottom={paddingBottom}
      paddingLeft={paddingLeft}
      paddingRight={paddingRight}
    >
      {children}
    </Root>
  );
}

export default TopComponent;
