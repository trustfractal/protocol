import styled from "styled-components";

import { ICredential } from "@pluginTypes/index";

import Text, {
  TextHeights,
  TextSizes,
  TextWeights,
} from "@popup/components/common/Text";
import Icon, { IconNames } from "@popup/components/common/Icon";
import LevelIcon from "@popup/components/common/LevelIcon";

const RootContainer = styled.div`
  display: flex;
  flex-direction: row;

  border-radius: var(--s-12);
  padding: var(--s-20) var(--s-12);

  background: var(--c-gray);

  color: var(--c-dark-blue);

  border: 1px solid rgba(19, 44, 83, 0.2);
`;

const LevelStatusContainer = styled.div`
  display: flex;
`;
const LevelContainer = styled.div`
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: flex-start;
`;
const StatusContainer = styled.div`
  flex: 1;
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
`;

const NameBadgesContainer = styled.div`
  display: flex;
  margin-top: var(--s-12);
`;
const NameContainer = styled.div`
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: flex-start;
`;

const LevelName = styled.div`
  opacity: 0.6;
`;

const LevelIconContainer = styled.div`
  margin-right: var(--s-12);
`;

const CredentialWrapper = styled.div`
  flex: 1;
  display: flex;
  flex-direction: column;
`;

const StatusName = styled.div`
  opacity: 0.6;
  margin-right: var(--s-8);
`;

const Status = styled.div`
  display: flex;
  align-items: center;
`;

type CredentialProps = {
  credential: ICredential;
};

function Credential(props: CredentialProps & React.HTMLProps<HTMLDivElement>) {
  const { credential } = props;

  const {
    properties: { full_name: name },
  } = credential;

  let hasName = true;
  if (name === undefined || (name as String).length === 0) {
    hasName = false;
  }

  const [level, ...addons] = credential.level.split("+");
  let levelName;

  const addonsStr = addons.join(" + ");

  if (level === "basic") {
    levelName = `ID Basic + ${addonsStr}`;
  } else {
    levelName = `ID Plus + ${addonsStr}`;
  }

  return (
    <RootContainer>
      <LevelIconContainer>
        <LevelIcon level={level} />
      </LevelIconContainer>
      <CredentialWrapper>
        <LevelStatusContainer>
          <LevelContainer>
            <Text height={TextHeights.LARGE} weight={TextWeights.BOLD}>
              {levelName}
            </Text>
          </LevelContainer>
          <StatusContainer>
            <Status>
              <StatusName>
                <Text size={TextSizes.SMALL} height={TextHeights.SMALL}>
                  Valid
                </Text>
              </StatusName>
              <Icon name={IconNames.VALID} />
            </Status>
          </StatusContainer>
        </LevelStatusContainer>
        <NameBadgesContainer>
          <NameContainer>
            {hasName && (
              <LevelName>
                <Text size={TextSizes.SMALL} height={TextHeights.SMALL}>
                  {name}
                </Text>
              </LevelName>
            )}
          </NameContainer>
        </NameBadgesContainer>
      </CredentialWrapper>
    </RootContainer>
  );
}

export default Credential;
