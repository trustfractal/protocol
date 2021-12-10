
import Button from "@common/Button";
import PasswordInput from "@common/PasswordInput";
import {
  Cta,
  Icon,
  IconNames,
  Title,
  VerticalSequence,
} from "@components/Common";
import { getProtocolService } from "@services/Factory";
import { useLoadedState } from "@utils/ReactHooks";
import { useState } from "react";
import styled from "styled-components";

const FCL_UNIT = BigInt(10 ** 12);

export function SweepTokens(props: { onFinish: () => void }) {
  const [page, setPage] = useState<JSX.Element | null>(null);

  const specifySend = (
    <SpecifyFrom
      onComplete={(hash) => {
        setPage(<SendComplete onFinish={props.onFinish} hash={hash} />);
      }}
      onCancel={props.onFinish}
    />
  );

  return page || specifySend;
}

function SpecifyFrom(props: {
  onComplete: (hash: string) => void;
  onCancel: () => void;
}) {
  const [mnemonic, setMnemonic] = useState("");
  const [loading, setLoading] = useState(false);

  const address = (() => {
    try {
      return getProtocolService().createSigner(mnemonic).address;
    } catch {
      return null;
    }
  })();

  const balance = useLoadedState(async () => {
    if (!address) return null;

    return await getProtocolService().getBalance(address);
  }, [address]);

  const amount = balance
    .map((b) => {
      if (b == null) return "?";
      return (b.free.toBigInt() / FCL_UNIT).toString();
    })
    .unwrapOrDefault("?");

  const isValid = address != null;

  const doSend = async () => {
    setLoading(true);
    const hash = await getProtocolService().sweepFromMnemonic(mnemonic);
    setLoading(false);
    props.onComplete(hash);
  };

  return (
    <ScreenContainer>
      <VerticalSequence>
        <Icon name={IconNames.PROTOCOL} />
        <Title>Sweep Funds from Address</Title>

        <PasswordInput
          className="password"
          label="From Mnemonic"
          value={mnemonic}
          error={
            isValid || mnemonic.length === 0 ? undefined : "Invalid mnemonic"
          }
          spellCheck="false"
          autoFocus
          onChange={(e: any) => setMnemonic(e.target.value)}
        />

        {isValid ? (
          <>
            <p>
              Will transfer <strong>{amount} FCL</strong> (all funds) from
              address
            </p>
            <BreakStrong>{address}</BreakStrong>
          </>
        ) : null}

        <HorizontalContainer>
          <Button alternative loading={loading} onClick={props.onCancel}>
            Cancel
          </Button>
          <Cta loading={loading} disabled={!isValid} onClick={doSend}>
            Sweep
          </Cta>
        </HorizontalContainer>
      </VerticalSequence>
    </ScreenContainer>
  );
}

const HorizontalContainer = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: center;
  align-items: flex-start;

  column-gap: var(--s-12);
`;

const ScreenContainer = styled.div`
  padding: var(--s-12);

  .password {
    align-self: stretch;
  }
`;

const BreakStrong = styled.strong`
  word-break: break-all;
  text-align: center;
`;

function SendComplete(props: { hash: string; onFinish: () => void }) {
  return (
    <ScreenContainer>
      <VerticalSequence>
        <Icon name={IconNames.PROTOCOL} />
        <Title>Transfer Complete</Title>

        <p>Transaction ID</p>
        <p>
          <BreakStrong>{props.hash}</BreakStrong>
        </p>

        <Cta onClick={props.onFinish}>Return</Cta>
      </VerticalSequence>
    </ScreenContainer>
  );
}
