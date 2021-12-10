
import Button from "@common/Button";
import Input from "@common/Input";
import { decodeAddress, encodeAddress } from "@polkadot/keyring";
import {
  Cta,
  Icon,
  IconNames,
  Title,
  VerticalSequence,
} from "@popup/components/Common";
import { getProtocolService } from "@services/Factory";
import { useState } from "react";
import styled from "styled-components";

const FCL_UNIT = BigInt(10 ** 12);

export function SendTokens(props: { onFinish: () => void }) {
  const [page, setPage] = useState<"specify" | "confirm" | JSX.Element>(
    "specify",
  );

  const [amount, setAmount] = useState(BigInt(0));
  const [destination, setDestination] = useState("");

  const [loading, setLoading] = useState(false);

  const specifySend = (
    <SpecifySend
      address={destination}
      onChangeAddress={setDestination}
      amount={amount}
      onChangeAmount={setAmount}
      onContinue={() => setPage("confirm")}
    />
  );

  const confirmSend = (
    <ConfirmSend
      address={destination}
      amount={amount}
      onConfirm={async () => {
        setLoading(true);
        const hash = await getProtocolService().sendToAddress(
          destination,
          amount,
        );
        setPage(<SendComplete onFinish={props.onFinish} hash={hash} />);
        setLoading(false);
      }}
      onCancel={() => setPage("specify")}
      loading={loading}
    />
  );

  if (page === "specify") {
    return specifySend;
  } else if (page === "confirm") {
    return confirmSend;
  } else {
    return page;
  }
}

function SpecifySend(props: {
  address: string;
  onChangeAddress: (a: string) => void;
  amount: bigint;
  onChangeAmount: (a: bigint) => void;
  onContinue: () => void;
}) {
  const addressError = getAddressError(props.address);
  const validAmount = props.amount > BigInt(0);
  const isValid = addressError == null && validAmount;

  return (
    <ScreenContainer>
      <VerticalSequence>
        <Icon name={IconNames.PROTOCOL} />
        <Title>Send to Address</Title>

        <HorizontalContainer>
          <Input
            label="Destination"
            value={props.address}
            error={props.address.length === 0 ? undefined : addressError}
            spellCheck="false"
            onChange={(e) => props.onChangeAddress(e.target.value)}
          />
          <Input
            label="Amount"
            type="number"
            value={
              props.amount === BigInt(0)
                ? ""
                : (props.amount / FCL_UNIT).toString()
            }
            onChange={(e) => {
              props.onChangeAmount(BigInt(e.target.value) * FCL_UNIT);
            }}
          />
        </HorizontalContainer>

        {isValid ? (
          <>
            <p>
              Will send{" "}
              <strong>{(props.amount / FCL_UNIT).toString()} FCL</strong> to
              address
            </p>
            <BreakStrong>{props.address}</BreakStrong>
          </>
        ) : null}

        <Cta disabled={!isValid} onClick={props.onContinue}>
          Send
        </Cta>
      </VerticalSequence>
    </ScreenContainer>
  );
}

function getAddressError(address: string): string | undefined {
  const isEthAddress = address.startsWith("0x");
  if (isEthAddress) return "Can only send to Substrate addresses (5FcL...)";

  if (!isValidAddress(address)) return "Invalid address";

  return;
}

function isValidAddress(address: string) {
  try {
    encodeAddress(decodeAddress(address));
    return true;
  } catch {
    return false;
  }
}

function ConfirmSend(props: {
  address: string;
  amount: bigint;
  onConfirm: () => void;
  onCancel: () => void;
  loading: boolean;
}) {
  return (
    <ScreenContainer>
      <VerticalSequence>
        <Icon name={IconNames.PROTOCOL} />
        <Title>Confirm Send</Title>

        <p>
          Send <strong>{(props.amount / FCL_UNIT).toString()} FCL</strong> to
        </p>
        <BreakStrong>{props.address}</BreakStrong>

        <HorizontalContainer>
          <Button alternative loading={props.loading} onClick={props.onCancel}>
            Cancel
          </Button>
          <Cta loading={props.loading} onClick={props.onConfirm}>
            Send
          </Cta>
        </HorizontalContainer>
      </VerticalSequence>
    </ScreenContainer>
  );
}

const ScreenContainer = styled.div`
  padding: var(--s-12);
`;

const HorizontalContainer = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: center;
  align-items: flex-start;

  column-gap: var(--s-12);
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
        <Title>Send Complete</Title>

        <p>Transaction ID</p>
        <p>
          <BreakStrong>{props.hash}</BreakStrong>
        </p>

        <Cta onClick={props.onFinish}>Return</Cta>
      </VerticalSequence>
    </ScreenContainer>
  );
}
