import React from "react";

import styled, { css } from "styled-components";

const { default: Check } = require("@assets/check.svg");
const { default: ChevronRight } = require("@assets/chevron-right.svg");
const { default: ChevronLeft } = require("@assets/chevron-left.svg");
const { default: ChevronDown } = require("@assets/chevron-down.svg");
const { default: Contacted } = require("@assets/contacted.svg");
const { default: Eye } = require("@assets/eye.svg");
const { default: EyeSlash } = require("@assets/eye-slash.svg");
const { default: LogoName } = require("@assets/logo-name.svg");
const { default: Logo } = require("@assets/logo.svg");
const { default: LogoSmall } = require("@assets/logo-small.svg");
const { default: Success } = require("@assets/success.svg");
const { default: Connected } = require("@assets/connected.svg");
const { default: Robot } = require("@assets/robot.svg");
const { default: CheckOutline } = require("@assets/check-outline.svg");
const { default: IDBasicSmall } = require("@assets/id-basic-small.svg");
const { default: IDBasic } = require("@assets/id-basic.svg");
const { default: IDPlusSmall } = require("@assets/id-plus-small.svg");
const { default: IDPlus } = require("@assets/id-plus.svg");
const { default: Valid } = require("@assets/valid.svg");
const { default: Invalid } = require("@assets/invalid.svg");
const { default: Pending } = require("@assets/pending.svg");
const { default: FractalToken } = require("@assets/fractal-token.svg");
const { default: FractalEthToken } = require("@assets/fractal-eth-token.svg");
const { default: MenuActive } = require("@assets/menu-active.svg");
const { default: MenuInactive } = require("@assets/menu-inactive.svg");
const { default: Export } = require("@assets/export.svg");
const { default: Refresh } = require("@assets/refresh.svg");
const { default: Import } = require("@assets/import.svg");
const { default: Issuing } = require("@assets/issuing.svg");
const { default: Unknown } = require("@assets/unknown.svg");
const { default: RequestAccepted } = require("@assets/request-accepted.svg");
const { default: RequestPending } = require("@assets/request-pending.svg");
const { default: RequestDeclined } = require("@assets/request-declined.svg");
const { default: Accepted } = require("@assets/accepted.svg");
const { default: Declined } = require("@assets/declined.svg");
const { default: About } = require("@assets/about.svg");
const { default: FractalFullLogo } = require("@assets/fractal-full-logo.svg");
const { default: Welcome } = require("@assets/welcome.svg");
const { default: Protocol } = require("@assets/protocol.svg");
const {
  default: ProtocolSetupSuccess,
} = require("@assets/protocol-setup-success.svg");
const {
  default: ProtocolSetupFailure,
} = require("@assets/protocol-setup-failure.svg");
const {
  default: ProtocolSetupPending,
} = require("@assets/protocol-setup-pending.svg");

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
  CHECK = "check",
  CHEVRON_RIGHT = "chevron-right",
  CHEVRON_LEFT = "chevron-left",
  CHEVRON_DOWN = "chevron-down",
  CONTACTED = "contacted",
  EYE = "eye",
  EYE_SLASH = "eye-slash",
  LOGO = "logo",
  LOGO_SMALL = "logo-small",
  LOGO_NAME = "logo-name",
  LEGACY_BADGE = "legacy-badge",
  SUCCESS = "success",
  CONNECTED = "connected",
  ROBOT = "robot",
  CHECK_OUTLINE = "check-outline",
  ID_BASIC_SMALL = "id-basic-small",
  ID_BASIC = "id-basic",
  ID_PLUS_SMALL = "id-plus-small",
  ID_PLUS = "id-plus",
  VALID = "valid",
  INVALID = "invalid",
  PENDING = "pending",
  FRACTAL_TOKEN = "fractal-token",
  FRACTAL_ETH_TOKEN = "fractal-eth-token",
  MENU_ACTIVE = "menu-active",
  MENU_INACTIVE = "menu-inactive",
  EXPORT = "export",
  IMPORT = "import",
  ISSUING = "issuing",
  UNKNOWN = "unknown",
  REFRESH = "refresh",
  REQUEST_ACCEPTED = "request-accepted",
  REQUEST_PENDING = "request-pending",
  REQUEST_DECLINED = "request-declined",
  ACCEPTED = "accepted",
  DECLINED = "declined",
  ABOUT = "about",
  FRACTAL_FULL_LOGO = "fractal-full-logo",
  WELCOME = "welcome",
  PROTOCOL = "protocol",
  PROTOCOL_SETUP_SUCCESS = "protocol-setup-success",
  PROTOCOL_SETUP_FAILURE = "protocol-setup-failure",
  PROTOCOL_SETUP_PENDING = "protocol-setup-pending",
}

const Icons: Record<string, any> = {
  [IconNames.CHECK]: Check,
  [IconNames.CHEVRON_RIGHT]: ChevronRight,
  [IconNames.CHEVRON_LEFT]: ChevronLeft,
  [IconNames.CHEVRON_DOWN]: ChevronDown,
  [IconNames.CONNECTED]: Connected,
  [IconNames.EYE]: Eye,
  [IconNames.EYE_SLASH]: EyeSlash,
  [IconNames.LOGO_NAME]: LogoName,
  [IconNames.LOGO]: Logo,
  [IconNames.LOGO_SMALL]: LogoSmall,
  [IconNames.SUCCESS]: Success,
  [IconNames.CONTACTED]: Contacted,
  [IconNames.ROBOT]: Robot,
  [IconNames.CHECK_OUTLINE]: CheckOutline,
  [IconNames.ID_BASIC_SMALL]: IDBasicSmall,
  [IconNames.ID_BASIC]: IDBasic,
  [IconNames.ID_PLUS_SMALL]: IDPlusSmall,
  [IconNames.ID_PLUS]: IDPlus,
  [IconNames.VALID]: Valid,
  [IconNames.INVALID]: Invalid,
  [IconNames.PENDING]: Pending,
  [IconNames.FRACTAL_TOKEN]: FractalToken,
  [IconNames.FRACTAL_ETH_TOKEN]: FractalEthToken,
  [IconNames.MENU_ACTIVE]: MenuActive,
  [IconNames.MENU_INACTIVE]: MenuInactive,
  [IconNames.EXPORT]: Export,
  [IconNames.IMPORT]: Import,
  [IconNames.ISSUING]: Issuing,
  [IconNames.UNKNOWN]: Unknown,
  [IconNames.REFRESH]: Refresh,
  [IconNames.REQUEST_ACCEPTED]: RequestAccepted,
  [IconNames.REQUEST_PENDING]: RequestPending,
  [IconNames.REQUEST_DECLINED]: RequestDeclined,
  [IconNames.ACCEPTED]: Accepted,
  [IconNames.DECLINED]: Declined,
  [IconNames.ABOUT]: About,
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
