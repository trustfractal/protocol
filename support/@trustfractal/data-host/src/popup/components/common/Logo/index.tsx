import React from "react";

import styled, { css } from "styled-components";
import Icon, { IconNames } from "@popup/components/common/Icon";

const Root = styled.div<{ height: string; width: string; clickable?: boolean }>`
  background: var(--c-white);
  border-radius: 50%;

  ${(props) =>
    props.clickable &&
    css`
      cursor: pointer;
    `}

  ${(props) => css`
    width: ${props.width};
    height: ${props.height};
  `}

  display: flex;
  justify-content: center;
  align-items: center;
`;

export enum LogoSizes {
  SMALL = "small",
  MEDIUM = "medium",
}

type LogoProps = {
  clickable?: boolean;
  width?: string;
  height?: string;
  size: LogoSizes;
};

const Sizes = {
  [LogoSizes.SMALL]: {
    name: IconNames.LOGO_SMALL,
    container: {
      width: "32px",
      height: "32px",
    },
    logo: {
      width: "20px",
      height: "16px",
    },
  },
  [LogoSizes.MEDIUM]: {
    name: IconNames.LOGO,
    container: {
      width: "80px",
      height: "80px",
    },
    logo: {
      width: "46px",
      height: "40px",
    },
  },
};

Logo.defaultProps = {
  clickable: false,
  size: LogoSizes.MEDIUM,
};

function Logo(props: LogoProps & React.HtmlHTMLAttributes<HTMLImageElement>) {
  const { size, ...otherProps } = props;
  const { name, logo, container } = Sizes[size];

  return (
    <Root {...container} {...otherProps}>
      <Icon name={name} {...logo} />
    </Root>
  );
}

export default Logo;
