import React from "react";

import styled from "styled-components";
import Icon, { IconNames } from "@popup/components/common/Icon";
import RequestsStatus from "@models/Request/status";

const RootContainer = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
`;

const IconContainer = styled.div`
  background: linear-gradient(
    180deg,
    #ffffff 18.75%,
    rgba(255, 255, 255, 0.4) 100%
  );
  border-radius: 50%;
  width: 80px;
  height: 80px;

  display: flex;
  justify-content: center;
  align-items: center;
`;

type RequestIconProps = {
  requester: string;
  status: string;
};

function RequestIcon(props: RequestIconProps) {
  const { requester, status } = props;

  return (
    <RootContainer>
      <IconContainer>
        <img src={requester} alt="requester" width="40px" height="40px" />
      </IconContainer>
      {status === RequestsStatus.ACCEPTED && (
        <Icon name={IconNames.REQUEST_ACCEPTED} />
      )}
      {status === RequestsStatus.PENDING && (
        <Icon name={IconNames.REQUEST_PENDING} />
      )}
      {status === RequestsStatus.DECLINED && (
        <Icon name={IconNames.REQUEST_DECLINED} />
      )}
      <IconContainer>
        <Icon name={IconNames.LOGO} />
      </IconContainer>
    </RootContainer>
  );
}

export default RequestIcon;
