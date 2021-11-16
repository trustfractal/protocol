import React from "react";

import Spinner from "@popup/components/common/Spinner";
import styled, { css } from "styled-components";

const Root = styled.button<{
  alternative: boolean;
  disabled: boolean;
}>`
  color: var(--c-white);
  background: var(--c-orange);
  border-radius: var(--s-12);
  padding: var(--s-14) var(--s-35);
  font-weight: bold;
  transition: color 0.2s ease-in-out, background-color 0.2s ease-in-out;

  display: flex;
  flex-direction: row;

  align-items: center;
  justify-content: center;

  min-width: var(--s-48);

  ${(props) =>
    props.disabled &&
    css`
      opacity: 0.6;
      cursor: default;
    `}

  ${(props) =>
    !props.disabled &&
    css`
      :hover {
        background: var(--c-dark-orange);
      }
    `}

  ${(props) =>
    props.alternative &&
    css`
      color: var(--c-orange);
      background: var(--c-transparent);
      border: 1px solid var(--c-orange);
    `}

    ${(props) =>
    props.alternative &&
    !props.disabled &&
    css`
      :hover {
        background: var(--c-orange);
        color: var(--c-white);
      }
    `}
`;

const LeftIconContainer = styled.div`
  margin-right: var(--s-12);
  display: flex;
`;

const RightIconContainer = styled.div`
  margin-left: var(--s-12);
  display: flex;
`;

export type ButtonProps = {
  loading: boolean;
  alternative: boolean;
  leftIcon?: JSX.Element;
  rightIcon?: JSX.Element;
};

Button.defaultProps = {
  loading: false,
  alternative: false,
};

export function Button(
  props: ButtonProps & React.ButtonHTMLAttributes<HTMLButtonElement>,
) {
  const { loading, disabled, children, leftIcon, rightIcon, ...otherProps } =
    props;

  return (
    <Root disabled={disabled || loading} {...otherProps}>
      {loading && (
        <LeftIconContainer>
          <Spinner alternative={otherProps.alternative} />
        </LeftIconContainer>
      )}
      {leftIcon !== undefined && (
        <LeftIconContainer>{leftIcon}</LeftIconContainer>
      )}
      {children}
      {rightIcon !== undefined && (
        <RightIconContainer>{rightIcon}</RightIconContainer>
      )}
    </Root>
  );
}

export default Button;
