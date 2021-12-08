import FractalFullLogo from "@assets/fractal-full-logo.svg";
import LogoName from "@assets/logo-name.svg";
import LogoSmall from "@assets/logo-small.svg";
import Logo from "@assets/logo.svg";
import ProtocolSetupFailure from "@assets/protocol-setup-failure.svg";
import ProtocolSetupPending from "@assets/protocol-setup-pending.svg";
import ProtocolSetupSuccess from "@assets/protocol-setup-success.svg";
import Protocol from "@assets/protocol.svg";
import Welcome from "@assets/welcome.svg";
import React from "react";
import styled, { css } from "styled-components";

const Root = styled.div<{
  clickable: boolean;
}>`
  ${(props) =>
    props.clickable &&
    css`
      cursor: pointer;
    `}
`;

export enum IconNames {
  LOGO = "logo",
  LOGO_SMALL = "logo-small",
  LOGO_NAME = "logo-name",
  FRACTAL_FULL_LOGO = "fractal-full-logo",
  WELCOME = "welcome",
  PROTOCOL = "protocol",
  PROTOCOL_SETUP_SUCCESS = "protocol-setup-success",
  PROTOCOL_SETUP_FAILURE = "protocol-setup-failure",
  PROTOCOL_SETUP_PENDING = "protocol-setup-pending",
}

const Icons: Record<string, any> = {
  [IconNames.LOGO_NAME]: LogoName,
  [IconNames.LOGO]: Logo,
  [IconNames.LOGO_SMALL]: LogoSmall,
  [IconNames.FRACTAL_FULL_LOGO]: FractalFullLogo,
  [IconNames.WELCOME]: Welcome,
  [IconNames.PROTOCOL]: Protocol,
  [IconNames.PROTOCOL_SETUP_SUCCESS]: ProtocolSetupSuccess,
  [IconNames.PROTOCOL_SETUP_FAILURE]: ProtocolSetupFailure,
  [IconNames.PROTOCOL_SETUP_PENDING]: ProtocolSetupPending,
};

type IconProps = {
  name: string;
  clickable: boolean;
  width?: string;
  height?: string;
};

Icon.defaultProps = {
  clickable: false,
};

function Icon(props: IconProps & React.HtmlHTMLAttributes<HTMLImageElement>) {
  const { name, clickable, onClick, ...otherProps } = props;

  const SVG = Icons[name];

  return (
    <Root clickable={clickable} onClick={onClick}>
      <SVG alt={name} {...otherProps} />
    </Root>
  );
}

export default Icon;
