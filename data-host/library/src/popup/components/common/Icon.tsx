import ChevronLeft from "@assets/chevron-left.svg";
import ChevronRight from "@assets/chevron-right.svg";
import FractalFullLogo from "@assets/fractal-full-logo.svg";
import IDBasicSmall from "@assets/id-basic-small.svg";
import IDBasic from "@assets/id-basic.svg";
import IDPlusSmall from "@assets/id-plus-small.svg";
import IDPlus from "@assets/id-plus.svg";
import Invalid from "@assets/invalid.svg";
import LogoName from "@assets/logo-name.svg";
import LogoSmall from "@assets/logo-small.svg";
import Logo from "@assets/logo.svg";
import Pending from "@assets/pending.svg";
import ProtocolSetupFailure from "@assets/protocol-setup-failure.svg";
import ProtocolSetupPending from "@assets/protocol-setup-pending.svg";
import ProtocolSetupSuccess from "@assets/protocol-setup-success.svg";
import Protocol from "@assets/protocol.svg";
import Valid from "@assets/valid.svg";
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
  CHEVRON_RIGHT = "chevron-right",
  CHEVRON_LEFT = "chevron-left",
  LOGO = "logo",
  LOGO_SMALL = "logo-small",
  LOGO_NAME = "logo-name",
  FRACTAL_FULL_LOGO = "fractal-full-logo",
  WELCOME = "welcome",
  ID_BASIC_SMALL = "id-basic-small",
  ID_BASIC = "id-basic",
  ID_PLUS_SMALL = "id-plus-small",
  ID_PLUS = "id-plus",
  VALID = "valid",
  INVALID = "invalid",
  PENDING = "pending",
  PROTOCOL = "protocol",
  PROTOCOL_SETUP_SUCCESS = "protocol-setup-success",
  PROTOCOL_SETUP_FAILURE = "protocol-setup-failure",
  PROTOCOL_SETUP_PENDING = "protocol-setup-pending",
}

const Icons: Record<string, any> = {
  [IconNames.CHEVRON_RIGHT]: ChevronRight,
  [IconNames.CHEVRON_LEFT]: ChevronLeft,
  [IconNames.LOGO_NAME]: LogoName,
  [IconNames.LOGO]: Logo,
  [IconNames.LOGO_SMALL]: LogoSmall,
  [IconNames.FRACTAL_FULL_LOGO]: FractalFullLogo,
  [IconNames.WELCOME]: Welcome,
  [IconNames.ID_BASIC_SMALL]: IDBasicSmall,
  [IconNames.ID_BASIC]: IDBasic,
  [IconNames.ID_PLUS_SMALL]: IDPlusSmall,
  [IconNames.ID_PLUS]: IDPlus,
  [IconNames.VALID]: Valid,
  [IconNames.INVALID]: Invalid,
  [IconNames.PENDING]: Pending,
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
