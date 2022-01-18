import Text, { TextHeights, TextSizes } from '@common/Text';
import { Activated, Hero, HeroLink } from '@components/DataScreen/Hero';
import { getDataHost } from '@services/Factory';
import { useLoadedState } from '@utils/ReactHooks';
import { useState } from 'react';
import styled from 'styled-components';

interface PageView {
  url: string;
  timestampMs: number;
}

const ListContainer = styled.div`
  padding: 0 var(--s-12);
`;

function HistoryList({ n }: { n: number }) {
  const history = useLoadedState(async () => {
    const items = [];
    for await (const item of getDataHost().iterBack()) {
      if (item.pageView == null) continue;
      items.push(item.pageView);
      if (items.length >= n) break;
    }
    return items;
  }, [n]);

  const historyItems = history
    .unwrapOrDefault([])
    .map((item) => <HistoryListItem item={item} key={item.timestampMs} />);
  return <ListContainer>{historyItems}</ListContainer>;
}

const ItemContainer = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  align-items: stretch;
  text-align: start;

  padding: var(--s-12) 0;

  p {
    color: var(--c-dark-blue);
    white-space: nowrap;
  }

  :not(:first-child) {
    margin-top: var(--s-6);
  }

  .long {
    max-width: 320px;

    text-overflow: ellipsis;
    overflow: hidden;
  }

  .date {
    opacity: 0.6;
  }

  &:not(:last-child) {
    border-bottom: 1px solid var(--c-gray);
  }
`;

function HistoryListItem({ item }: { item: PageView }) {
  return (
    <ItemContainer>
      <Text className="date" size={TextSizes.SMALL} height={TextHeights.SMALL}>
        {new Date(item.timestampMs).toLocaleString()}
      </Text>
      <Text className="long" size={TextSizes.SMALL} height={TextHeights.SMALL}>
        Viewed <strong>{item.url}</strong>
      </Text>
    </ItemContainer>
  );
}

function WebpageViews() {
  const [historyOpen, setHistoryOpen] = useState<boolean>(false);
  const toggleHistory = () => setHistoryOpen(!historyOpen);

  return (
    <Hero title="Browsing History Data" callout={<Activated />}>
      {historyOpen ? (
        <>
          <HeroLink isExit onClick={toggleHistory} text="Close" />
          <HistoryList n={8} />
        </>
      ) : (
        <>
          <HeroLink onClick={toggleHistory} text="View History" />
          <HistoryList n={2} />
        </>
      )}
    </Hero>
  );
}

export default WebpageViews;
