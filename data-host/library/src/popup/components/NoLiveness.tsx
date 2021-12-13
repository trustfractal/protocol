import {
  BoldText,
  Cta,
  Icon,
  IconNames,
  Title,
  VerticalSequence,
} from "@components/Common";
import { environment } from "@popup/Environment";


export function NoLiveness({ onClick }: { onClick: () => void }) {
  return (
    <VerticalSequence>
      <Icon name={IconNames.PROTOCOL} />

      <Title>You haven’t verified your identity yet</Title>
      <>
      <BoldText>
        To earn {environment.PROTOCOL_CURRENCY}, start by providing a valid
        liveness.
      </BoldText>

      <Cta onClick={onClick}>Verify Identity</Cta>
    </>
    </VerticalSequence>
  );
}
