import { useState } from "react";
import styled, { css } from "styled-components";

import { IRequest } from "@pluginTypes/index";

import Request from "@models/Request";

import Text, {
  TextHeights,
  TextSizes,
  TextWeights,
} from "@popup/components/common/Text";
import Icon, { IconNames } from "@popup/components/common/Icon";
import moment from "moment";
import RequestsStatus from "@models/Request/status";

const RootContainer = styled.div`
  box-shadow: 0px 8px 12px #061a3a;
  border-bottom-left-radius: var(--s-12);
  border-bottom-right-radius: var(--s-12);
  margin-top: calc(-1 * var(--s-12));

  background-color: var(--c-white);
  color: var(--c-dark-blue);
  border-top: 1px solid rgb(19, 44, 83, 0.2);
`;

const ExpandContainer = styled.div<{ collapsed: boolean }>`
  user-select: none;
  cursor: pointer;

  display: flex;
  justify-content: space-between;
  align-items: center;

  padding: var(--s-20) var(--s-12);

  color: var(--c-white);
  background-color: var(--c-orange);

  ${(props) =>
    props.collapsed &&
    css`
      color: var(--c-orange);
      background-color: var(--c-white);

      border-bottom: 1px solid rgb(19, 44, 83, 0.2);
    `}
`;

const ExpandLabel = styled.div`
  text-transform: uppercase;
`;
const ExpandButton = styled.div``;
const LastActivityLabel = styled.div`
  opacity: 0.6;
  user-select: none;
  text-transform: uppercase;
`;
const LastActivity = styled.div`
  margin: var(--s-12) 0;
`;
const Activity = styled.div`
  margin-top: var(--s-12);
`;
const ActivityContent = styled.div``;
const ActivityIcon = styled.div``;
const ActivitiesContainer = styled.div``;
const ActivitiesLabel = styled.div`
  padding-top: var(--s-20);
  padding-bottom: var(--s-12);
  padding-left: var(--s-12);

  user-select: none;
  text-transform: uppercase;
`;
const ActivityDate = styled.div`
  opacity: 0.6;
  user-select: none;
`;
const ActivityContainer = styled.div`
  padding: var(--s-20) var(--s-12);

  display: flex;
  justify-content: space-between;
  align-items: center;

  :not(:last-child) {
    border-bottom: 1px solid rgb(19, 44, 83, 0.2);
  }
`;

type HistoryProps = {
  requests: IRequest[];
};

const NUMBER_OF_DISPLAYED_REQUESTS = 5;

function History(props: HistoryProps & React.HTMLProps<HTMLDivElement>) {
  const { requests } = props;

  const [collapsed, setCollapsed] = useState(true);
  const sortedRequests = requests
    .sort(Request.sortByUpdatedAt)
    .slice(0, NUMBER_OF_DISPLAYED_REQUESTS);
  const [lastRequest] = sortedRequests;

  const formatDate = (date: number) => moment(date).format("DD/MM/YY · h:mm A");

  const getRequestText = (request: IRequest) => {
    if (request.status === RequestsStatus.ACCEPTED) {
      return (
        <span>
          Allow <b>{request.requester.name}</b> to access credential
        </span>
      );
    }

    return (
      <span>
        Deny <b>{request.requester.name}</b> to access credential
      </span>
    );
  };

  const getRequestIcon = (request: IRequest) => {
    if (request.status === RequestsStatus.ACCEPTED) {
      return <Icon name={IconNames.ACCEPTED} />;
    }

    return <Icon name={IconNames.DECLINED} />;
  };

  return (
    <RootContainer>
      <ExpandContainer
        collapsed={collapsed}
        onClick={() => setCollapsed(!collapsed)}
      >
        {collapsed && (
          <>
            <ExpandLabel>
              <Text weight={TextWeights.SEMIBOLD}>View History</Text>
            </ExpandLabel>
            <ExpandButton>
              <Icon name={IconNames.CHEVRON_RIGHT} />
            </ExpandButton>
          </>
        )}
        {!collapsed && (
          <>
            <ExpandButton>
              <Icon name={IconNames.CHEVRON_LEFT} />
            </ExpandButton>
            <ExpandLabel>
              <Text weight={TextWeights.SEMIBOLD}>Close</Text>
            </ExpandLabel>
          </>
        )}
      </ExpandContainer>
      {collapsed && (
        <ActivityContainer>
          <ActivityContent>
            <LastActivityLabel>
              <Text size={TextSizes.SMALL} height={TextHeights.SMALL}>
                Last Activity
              </Text>
            </LastActivityLabel>
            <LastActivity>
              <Text size={TextSizes.SMALL} height={TextHeights.SMALL}>
                {getRequestText(lastRequest)}
              </Text>
            </LastActivity>
            <ActivityDate>
              <Text size={TextSizes.SMALL} height={TextHeights.SMALL}>
                {formatDate(lastRequest.updatedAt)}
              </Text>
            </ActivityDate>
          </ActivityContent>
          <ActivityIcon>{getRequestIcon(lastRequest)}</ActivityIcon>
        </ActivityContainer>
      )}
      {!collapsed && (
        <ActivitiesContainer>
          <ActivitiesLabel>
            <Text>Activity History</Text>
          </ActivitiesLabel>
          {sortedRequests.map((request) => (
            <ActivityContainer key={request.id}>
              <ActivityContent>
                <ActivityDate>
                  <Text size={TextSizes.SMALL} height={TextHeights.SMALL}>
                    {formatDate(request.updatedAt)}
                  </Text>
                </ActivityDate>
                <Activity>
                  <Text size={TextSizes.SMALL} height={TextHeights.SMALL}>
                    {getRequestText(request)}
                  </Text>
                </Activity>
              </ActivityContent>
              <ActivityIcon>{getRequestIcon(request)}</ActivityIcon>
            </ActivityContainer>
          ))}
        </ActivitiesContainer>
      )}
    </RootContainer>
  );
}

export default History;
