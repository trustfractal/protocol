import { Button } from '@common/Button';
import { IconNames } from '@common/Icon';
import Text, { TextHeights, TextSizes } from '@common/Text';
import { Activated, Hero } from '@components/DataScreen/Hero';
import { getMintingRegistrar, getProtocolService } from '@services/Factory';
import { MintingError as RegistrarMintingError } from '@services/MintingRegistrar';
import {
  MintingHistoryEvent,
  MintingReceived,
  MintingRegistered,
} from '@services/Protocol';
import { formatFloat } from '@utils/FormatUtils';
import { useLoadedState } from '@utils/ReactHooks';
import { useState } from 'react';
import { BarLoader as Loader } from 'react-spinners';
import ReactTooltip from 'react-tooltip';
import styled from 'styled-components';

export function Minting() {
  const isRegistered = useLoadedState(async () => {
    return await getProtocolService().isRegisteredForMinting();
  });
  const showRegisterButton = isRegistered.map((r) => !r).unwrapOrDefault(false);
  const callout = isRegistered
    .map((val) =>
      val ? (
        <Activated text="Registered" icon={IconNames.VALID} />
      ) : (
        <NotRegistered />
      )
    )
    .unwrapOrDefault(<Activated text="Loading" icon={IconNames.PENDING} />);

  const history = useLoadedState(async () => {
    return await getProtocolService().mintingHistory(4);
  });
  const historyItems = history
    .map((events) => {
      return events.map((event) => {
        return <HistoryItem key={event.at.getTime()} event={event} />;
      });
    })
    .unwrapOrDefault(
      <div className="loader">
        <Loader width={'100%'} color={'var(--c-orange)'} />
      </div>
    );

  const [registering, setRegistering] = useState(false);
  const tryRegister = async () => {
    try {
      setRegistering(true);
      await getMintingRegistrar().tryRegister();
    } finally {
      isRegistered.reload();
      setRegistering(false);
    }
  };

  return (
    <Hero title="Minting" callout={callout}>
      <HistoryContainer>
        {showRegisterButton ? (
          <Button onClick={tryRegister} loading={registering}>
            Register
          </Button>
        ) : null}
        {historyItems}
      </HistoryContainer>
    </Hero>
  );
}

const NotRegisteredContainer = styled.div`
  .text p {
    text-decoration: underline;
    text-decoration-style: dotted;
  }

  #notRegistered {
    opacity: 1;
  }
`;

function NotRegistered() {
  const error = useLoadedState(async () => {
    return await getMintingRegistrar().latestError();
  });

  const errorMessage = error
    // eslint-disable-next-line react/jsx-key
    .map((e) => <MintingError error={e} />)
    .unwrapOrDefault(<p>Loading</p>);

  return (
    <NotRegisteredContainer>
      <div className="text" data-tip data-for="notRegistered">
        <Activated text="Not Registered" icon={IconNames.INVALID} />
      </div>

      <ReactTooltip id="notRegistered" place="top" effect="solid">
        {errorMessage}
      </ReactTooltip>
    </NotRegisteredContainer>
  );
}

function MintingError({ error }: { error: RegistrarMintingError | null }) {
  if (error == null) {
    return <p>Registration has not been attempted.</p>;
  }
  if (error.type === 'unknown') {
    return (
      <>
        <p>Unhandled error:</p>
        <p>{error.message}</p>
      </>
    );
  }
  if (error.type === 'identity_registration') {
    return <p>Identity registration failed.</p>;
  }
  if (error.type === 'minting_registration') {
    return <p>Minting registration failed for unknown reasons.</p>;
  }
  if (error.type === 'cant_extend_dataset') {
    return <p>Could not extend existing dataset.</p>;
  }

  checkExhaustive(error);
}

function checkExhaustive(v: never): never {
  throw new Error(`Unhandled value ${v}`);
}

const HistoryContainer = styled.div`
  padding: var(--s-12);

  display: flex;
  flex-direction: column;

  > *:not(:last-child) {
    margin-bottom: var(--s-12);
  }

  .loader {
    align-self: stretch;

    display: flex;
  }
`;

function HistoryItem({ event }: { event: MintingHistoryEvent }) {
  const content = {
    received: (event: MintingReceived) => {
      return (
        <Text size={TextSizes.SMALL} height={TextHeights.SMALL}>
          Received{' '}
          <strong>{formatFloat(event.amount / 10 ** 12, 3)} FCL</strong>
        </Text>
      );
    },
    registered: (_event: MintingRegistered) => {
      return (
        <Text size={TextSizes.SMALL} height={TextHeights.SMALL}>
          Registered for minting
        </Text>
      );
    },
  }[event.kind](event as any);

  return (
    <HistoryItemContainer>
      {content}

      <Text className="date" size={TextSizes.SMALL} height={TextHeights.SMALL}>
        {event.at.toLocaleString()}
      </Text>
    </HistoryItemContainer>
  );
}

const HistoryItemContainer = styled.div`
  color: black;

  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;

  &:not(:last-child) {
    padding-bottom: var(--s-12);
    border-bottom: 1px solid var(--c-gray);
  }

  p {
    color: var(--c-dark-blue);
    white-space: nowrap;
  }

  .date {
    opacity: 0.6;
  }
`;
