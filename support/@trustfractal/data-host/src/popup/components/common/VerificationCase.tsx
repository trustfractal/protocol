import styled from "styled-components";

import { IVerificationCase } from "@pluginTypes/index";

import Text, {
  TextHeights,
  TextSizes,
  TextWeights,
} from "@popup/components/common/Text";
import Icon, { IconNames } from "@popup/components/common/Icon";
import LevelIcon from "@popup/components/common/LevelIcon";
import VerificationCaseStatus from "@models/VerificationCase/status";

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

const LevelIconContainer = styled.div`
  margin-right: var(--s-12);
  opacity: 0.2;
`;

const VerificationCaseWrapper = styled.div`
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

type VerificationCaseProps = {
  verificationCase: IVerificationCase;
};

function VerificationCase(
  props: VerificationCaseProps & React.HTMLProps<HTMLDivElement>,
) {
  const { verificationCase } = props;

  const [level, ...addons] = verificationCase.level.split("+");
  let levelName;

  const addonsStr = addons.join(" + ");

  if (level === "basic") {
    levelName = `ID Basic + ${addonsStr}`;
  } else {
    levelName = `ID Plus + ${addonsStr}`;
  }

  let statusIcon, statusName;

  switch (verificationCase.status) {
    case VerificationCaseStatus.PENDING:
      statusIcon = IconNames.PENDING;
      statusName = "Under review";
      break;

    case VerificationCaseStatus.CONTACTED:
      statusIcon = IconNames.CONTACTED;
      statusName = "Action required";
      break;

    case VerificationCaseStatus.ISSUING:
      statusIcon = IconNames.ISSUING;
      statusName = "Generating credential";
      break;

    default:
      statusIcon = IconNames.UNKNOWN;
      statusName = "";
  }

  return (
    <RootContainer>
      <LevelIconContainer>
        <LevelIcon level={level} />
      </LevelIconContainer>
      <VerificationCaseWrapper>
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
                  {statusName}
                </Text>
              </StatusName>
              <Icon name={statusIcon} />
            </Status>
          </StatusContainer>
        </LevelStatusContainer>
      </VerificationCaseWrapper>
    </RootContainer>
  );
}

export default VerificationCase;
