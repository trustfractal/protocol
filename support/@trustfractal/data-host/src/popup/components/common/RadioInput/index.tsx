import React from "react";

import styled from "styled-components";

const RadioIcon = styled.span`
  position: absolute;
  top: 0;
  left: 0;

  border-radius: var(--s-24);

  width: calc(var(--s-24) - 1px);
  height: calc(var(--s-24) - 1px);
  border: 1px solid var(--c-dark-blue);
  background-color: var(--c-white);

  :after {
    content: "";
    position: absolute;
    display: none;

    background-color: var(--c-orange);

    left: 3px;
    top: 3px;
    width: var(--s-16);
    height: var(--s-16);
    border-radius: var(--s-24);
  }
`;

const Radio = styled.input`
  position: absolute;
  opacity: 0;
  height: 0;
  width: 0;

  :checked ~ ${RadioIcon}:after {
    display: block;
  }
`;

const RootContainer = styled.div`
  user-select: none;
  cursor: pointer;
  position: relative;

  width: var(--s-24);
  height: var(--s-24);

  :hover ${Radio}:not(:checked) ~ ${RadioIcon} {
    background-color: var(--c-dark-gray);
  }
`;

function RadioInput(props: React.InputHTMLAttributes<HTMLInputElement>) {
  const { children, ...otherProps } = props;

  const ref = React.createRef<HTMLInputElement>();

  return (
    <RootContainer>
      <Radio ref={ref} type="radio" {...otherProps} />
      <RadioIcon onClick={() => ref.current?.click()} />
    </RootContainer>
  );
}

export default RadioInput;
