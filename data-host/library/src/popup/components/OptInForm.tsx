
import {
  BoldText,
  Cta,
  Icon,
  IconNames,
  Subsubtitle,
  Subtitle,
  VerticalSequence,
} from '@components/Common';
import { environment } from "@popup/Environment";
import styled from "styled-components";

const List = styled.ul`
  list-style: none;

  li {
    display: grid;
    grid-template-columns: 0 1fr;
    grid-gap: var(--s-24);
    align-items: start;
    font-size: var(--s-16);
    line-height: var(--s-24);
    text-align: start;

    &:not(:last-child) {
      margin-bottom: var(--s-24);
    }

    &::before {
      content: "🚀";
    }
  }
`;

export function OptInForm({ onOptIn }: { onOptIn: () => void }) {
  return (
    <VerticalSequence>
      <Icon name={IconNames.PROTOCOL} />

      <BoldText>
        Get ready for controlling and monetizing your own browsing data.
      </BoldText>

      <Subtitle uppercase>Wallet functionality</Subtitle>

      <List>
        <li>Tracks and stores your browsing data.</li>
        <li>The data is only stored on your local device. </li>
        <li>Publishes a proof of your data provisioning activity on chain</li>
        <li>
          You get rewards in {environment.PROTOCOL_CURRENCY} for storing your
          data.
        </li>
      </List>

      <Subsubtitle uppercase center>
        By clicking “Join the Protocol” you agree to our{" "}
        <a
          href={`${environment.FRACTAL_WEBSITE_URL}/documents/mainnet-launch-special-user-agreement`}
          target="_blank"
          rel="noreferrer"
        >
          Special Mainnet Launch User Agreement.
        </a>
      </Subsubtitle>

      <Cta onClick={onOptIn}>Join the Protocol</Cta>
    </VerticalSequence>
  );
}
