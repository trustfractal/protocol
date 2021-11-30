import Logo from "@common/Logo";
import Spinner from "@common/Spinner";
import Text, { TextHeights, TextSizes } from "@common/Text";
import environment from "@popup/Environment";
import { useEffect } from "react";
import styled from "styled-components";

const RootContainer = styled.div`
  min-width: 352px;
  min-height: 412px;

  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
`;

const LabelContainer = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: center;

  margin-top: var(--s-168);
`;

const SpinnerContainer = styled.div`
  margin-right: var(--s-12);
`;

function Loading() {
  useEffect(() => {
    const start = window.performance.now();
    return () => {
      if (!environment.IS_DEV) return;
      console.info(
        "Showed loading for",
        window.performance.now() - start,
        "ms",
      );
    };
  }, []);

  return (
    <RootContainer>
      <Logo />
      <LabelContainer>
        <SpinnerContainer>
          <Spinner alternative />
        </SpinnerContainer>
        <Text size={TextSizes.LARGE} height={TextHeights.LARGE}>
          Loading...
        </Text>
      </LabelContainer>
    </RootContainer>
  );
}

export default Loading;
