import {
  BoldText,
  Cta,
  Icon,
  IconNames,
  VerticalSequence,
} from '@components/Common';

export function ExtensionSetup({ onOptIn }: { onOptIn: () => void }) {
  return (
    <VerticalSequence>
      <Icon name={IconNames.PROTOCOL} />

      <BoldText>
        First time opening the Protocol tab, please follow the initialization
        process through the link.
      </BoldText>

      <Cta onClick={onOptIn}>Initialize the Protocol</Cta>
    </VerticalSequence>
  );
}
