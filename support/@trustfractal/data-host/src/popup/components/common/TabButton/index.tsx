import React from "react";

import styled, { css } from "styled-components";

import Text, { TextHeights, TextSizes } from "@popup/components/common/Text";

const RootContainer = styled.button<{
  index: number;
  total: number;
  selected: boolean;
  disabled: boolean;
}>`
  display: flex;
  flex: 1;
  flex-direction: column;
  align-items: center;
  justify-content: center;

  color: var(--c-orange);
  background: var(--c-transparent);
  border-bottom: 1px solid var(--c-orange);
  border-right: 1px solid var(--c-orange);
  border-left: 1px solid var(--c-orange);

  padding: var(--s-14) var(--s-35);

  ${(props) =>
    props.disabled &&
    css`
      opacity: 0.6;
      cursor: default;
    `}

  ${(props) =>
    props.selected &&
    css`
      font-weight: bold;
      color: var(--c-white);
      background: var(--c-orange);
    `}

  ${(props) =>
    props.index === 0 &&
    css`
      border-left-width: 0px;
    `}

  ${(props) =>
    props.index === props.total - 1 &&
    css`
      border-right-width: 0px;
    `}
`;

const ComingSoonContainer = styled.span`
  color: var(--c-white);
`;

type TabButtonProps = {
  disabled: boolean;
  selected: boolean;
  label: string;
  index: number;
  total: number;
};

TabButton.defaultProps = {
  disabled: false,
};

function TabButton(
  props: TabButtonProps & React.ButtonHTMLAttributes<HTMLButtonElement>,
) {
  const { index, total, label, disabled, selected, ...otherProps } = props;

  return (
    <RootContainer
      index={index}
      total={total}
      disabled={disabled}
      selected={selected}
      {...otherProps}
    >
      {label}
      {disabled && (
        <ComingSoonContainer>
          <Text size={TextSizes.SMALL} height={TextHeights.SMALL}>
            Coming Soon
          </Text>
        </ComingSoonContainer>
      )}
    </RootContainer>
  );
}

export default TabButton;
