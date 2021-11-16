import React from "react";

import styled, { css, keyframes } from "styled-components";

const rotation = keyframes`
  0% {
    -webkit-transform: rotate(0deg);
    transform: rotate(0deg);
  }
  100% {
    -webkit-transform: rotate(360deg);
    transform: rotate(360deg);
  }
`;

const Root = styled.p<SpinnerProps>`
  width: var(--s-16);
  height: var(--s-16);
  border-radius: 50%;
  background: var(--c-white);
  background: linear-gradient(
    to right,
    var(--c-white) 10%,
    var(--c-transparent) 42%
  );

  ${(props) =>
    props.alternative &&
    css`
      background: var(--c-orange);
      background: linear-gradient(
        to right,
        var(--c-orange) 10%,
        var(--c-transparent) 42%
      );
    `}

  position: relative;
  -webkit-animation: ${rotation} 1.4s infinite linear;
  animation: ${rotation} 1.4s infinite linear;
  -webkit-transform: translateZ(0);
  -ms-transform: translateZ(0);
  transform: translateZ(0);

  :before {
    width: 50%;
    height: 50%;
    background: var(--c-white);
    ${(props) =>
      props.alternative &&
      css`
        background: var(--c-orange);
      `}
    border-radius: 100% 0 0 0;
    position: absolute;
    top: 0;
    left: 0;
    content: "";
  }

  :after {
    background: var(--c-orange);
    ${(props) =>
      props.alternative &&
      css`
        background: var(--c-dark-blue);
      `}
    width: 75%;
    height: 75%;
    border-radius: 50%;
    content: "";
    margin: auto;
    position: absolute;
    top: 0;
    left: 0;
    bottom: 0;
    right: 0;
  }
`;

type SpinnerProps = {
  alternative: boolean;
};

Spinner.defaultProps = {
  alternative: false,
};

function Spinner(props: SpinnerProps & React.HTMLAttributes<HTMLDivElement>) {
  const { children, ...otherProps } = props;

  return <Root {...otherProps}>{children}</Root>;
}

export default Spinner;
