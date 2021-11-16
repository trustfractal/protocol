import React from "react";
import styled, { css } from "styled-components";

import Icon, { IconNames } from "@popup/components/common/Icon";

const RootContainer = styled.div<{ icon: IconNames }>`
  border-radius: 100%;

  ${(props) =>
    props.icon === IconNames.ID_BASIC &&
    css`
      background: radial-gradient(
        57.81% 68.75% at 23.44% 15.62%,
        #a5c8ff 5.21%,
        #4073c2 44.35%,
        #132c53 87.5%,
        #00122f 100%
      );
      box-shadow: 0px 8px 12px #aabdda;
    `}

  ${(props) =>
    props.icon === IconNames.ID_BASIC_SMALL &&
    css`
      background: radial-gradient(
        57.81% 68.75% at 23.44% 15.62%,
        #a5c8ff 5.21%,
        #4073c2 44.35%,
        #132c53 87.5%,
        #00122f 100%
      );
      box-shadow: 0px 4px 8px #aabdda;
    `}

    ${(props) =>
    props.icon === IconNames.ID_PLUS_SMALL &&
    css`
      background: radial-gradient(
        57.81% 68.75% at 23.44% 15.62%,
        #ffeadf 10.04%,
        #ffc6aa 44.35%,
        #ff671d 87.5%,
        #e14a00 100%
      );
      box-shadow: 0px 4px 8px #ffc4a8;
    `}

  ${(props) =>
    props.icon === IconNames.ID_PLUS &&
    css`
      background: radial-gradient(
        57.81% 68.75% at 23.44% 15.62%,
        #ffeadf 10.04%,
        #ffc6aa 44.35%,
        #ff671d 87.5%,
        #e14a00 100%
      );
      box-shadow: 0px 8px 12px #ffc4a8;
    `}
`;

export enum LevelIconSizes {
  SMALL = "small",
  MEDIUM = "medium",
}

type LevelIconProps = {
  level: string;
  size?: LevelIconSizes;
};

LevelIcon.defaultProps = {
  size: LevelIconSizes.MEDIUM,
};

function LevelIcon(
  props: LevelIconProps & React.HtmlHTMLAttributes<HTMLImageElement>,
) {
  const { level, size } = props;
  let icon;

  if (level === "basic") {
    if (size === LevelIconSizes.SMALL) {
      icon = IconNames.ID_BASIC_SMALL;
    } else {
      icon = IconNames.ID_BASIC;
    }
  } else {
    if (size === LevelIconSizes.SMALL) {
      icon = IconNames.ID_PLUS_SMALL;
    } else {
      icon = IconNames.ID_PLUS;
    }
  }

  return (
    <RootContainer icon={icon}>
      <Icon name={icon} {...props} />
    </RootContainer>
  );
}

export default LevelIcon;
