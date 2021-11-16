import styled from "styled-components";

import Button, { ButtonProps } from "@popup/components/common/Button";
import Text, { TextWeights } from "@popup/components/common/Text";
import { Subsubtitle } from "@popup/components/common/Subtitle";

export { default as Text } from "@popup/components/common/Text";
export {
  default as Subtitle,
  Subsubtitle,
} from "@popup/components/common/Subtitle";
export { default as Icon, IconNames } from "@popup/components/common/Icon";
export { default as Input } from "@popup/components/common/Input";
export { default as Title } from "@popup/components/common/Title";

const Container = styled.div`
  width: 100%;
  height: 100%;

  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  text-align: center;

  > *:not(:last-child) {
    margin-bottom: var(--s-16);
  }
`;

const Link = styled.a`
  cursor: pointer;
  color: var(--c-orange);
  text-decoration: underline;
`;

export const ClickableText = styled.button`
  outline: none;
  background: none;
  margin: 0;
  padding: 0;
  color: inherit;
`;

export function VerticalSequence(props: React.HTMLProps<HTMLDivElement>) {
  return (
    <Container>
      {props.children}

      <Subsubtitle center style={{ alignSelf: "flex-end" }}>
        If you need help on anything related to Fractal ID Wallet, please
        contact us at{" "}
        <Link href="mailto:support@fractal.id">support@fractal.id</Link>
      </Subsubtitle>
    </Container>
  );
}

Cta.defaultProps = {
  loading: false,
  alternative: false,
};

export function Cta(
  props: React.ButtonHTMLAttributes<HTMLButtonElement> & ButtonProps,
) {
  const { children, ...other } = props;
  return <Button {...other}>{children}</Button>;
}

export function BoldText({
  children,
  center,
}: {
  children: React.ReactNode;
  center?: boolean;
}) {
  return (
    <Text weight={TextWeights.BOLD} center={center}>
      {children}
    </Text>
  );
}
