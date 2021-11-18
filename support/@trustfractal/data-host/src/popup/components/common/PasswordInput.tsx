import React, { useState } from "react";

import styled from "styled-components";
import Input, { InputProps } from "@popup/components/common/Input";
import Icon, { IconNames } from "@popup/components/common/Icon";

const Root = styled.div`
  position: relative;
`;

const IconContainer = styled.div`
  position: absolute;
  top: var(--s-19);
  right: 0;
  padding: var(--s-5);
  cursor: pointer;
`;

type PasswordInputProps = {
  defaultVisible: boolean;
};

PasswordInput.defaultProps = {
  defaultVisible: false,
};

function PasswordInput(
  props: PasswordInputProps &
    InputProps &
    React.InputHTMLAttributes<HTMLInputElement>,
) {
  const { children, defaultVisible, ...otherProps } = props;

  const [visible, setVisible] = useState(defaultVisible);

  return (
    <Root className={props.className}>
      <Input type={visible ? "text" : "password"} {...otherProps} />
      <IconContainer onClick={() => setVisible(!visible)}>
        <Icon name={visible ? IconNames.EYE : IconNames.EYE_SLASH} />
      </IconContainer>
    </Root>
  );
}

export default PasswordInput;
