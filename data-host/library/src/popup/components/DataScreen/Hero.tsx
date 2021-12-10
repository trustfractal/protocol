
import Icon, { IconNames } from "@common/Icon";
import LevelIcon from "@common/LevelIcon";
import Text, {
  TextHeights,
  TextSizes,
  TextWeights,
} from "@common/Text";
import styled from "styled-components";

export interface HeroProps {
  title: string;
  callout?: React.ReactNode;
  children?: React.ReactNode;
}

export function Hero(props: HeroProps) {
  return (
    <HeroListContainer>
      <Header>
        <HeaderInfo>
          <HeaderIcon>
            <LevelIcon level="plus" />
          </HeaderIcon>
          <Text
            style={{ color: "var(--c-dark-blue)" }}
            weight={TextWeights.BOLD}
          >
            {props.title}
          </Text>
        </HeaderInfo>

        <CalloutContainer>{props.callout}</CalloutContainer>
      </Header>

      <ListContentContainer>{props.children}</ListContentContainer>
    </HeroListContainer>
  );
}

const HeroListContainer = styled.div`
  width: 100%;

  flex-direction: column;
  align-items: stretch;
  justify-content: center;

  border-radius: var(--s-12);
  overflow: hidden;

  box-shadow: 0px var(--s-8) var(--s-12) #061a3a;
  background-color: var(--c-white);
`;

const Header = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;

  padding: var(--s-20) var(--s-12);

  background-color: var(--c-gray);

  border: 1px solid rgba(19, 44, 83, 0.2);
  border-bottom: none;
`;

const HeaderInfo = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
`;

const HeaderIcon = styled.div`
  margin-right: var(--s-12);
`;

const CalloutContainer = styled.div``;

const ActivationIcon = styled.div`
  margin-left: var(--s-4);
`;

const ListContentContainer = styled.div`
  display: flex;
  flex-direction: column;
  align-items: stretch;
`;

export function Activated({ text, icon }: { text?: string; icon?: string }) {
  return (
    <ActivatedContainer>
      <Text size={TextSizes.SMALL} height={TextHeights.SMALL}>
        {text || "Activated"}
      </Text>
      <ActivationIcon>
        <Icon name={icon || IconNames.VALID} width="24" height="24" />
      </ActivationIcon>
    </ActivatedContainer>
  );
}

const ActivatedContainer = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;

  color: var(--c-dark-blue);
`;

export interface HeroLinkProps {
  text: string;
  isExit?: boolean;
  onClick?: () => void;
}

export function HeroLink(props: HeroLinkProps) {
  return (
    <LinkContainer isExit={props.isExit} onClick={props.onClick}>
      <Text weight={TextWeights.SEMIBOLD}>{props.text}</Text>

      <Icon
        name={props.isExit ? IconNames.CHEVRON_LEFT : IconNames.CHEVRON_RIGHT}
      />
    </LinkContainer>
  );
}

const LinkContainer = styled.button<{ isExit?: boolean }>`
  display: flex;
  flex-direction: ${(props) => (props.isExit ? "row-reverse" : "row")};
  align-items: center;
  justify-content: space-between;

  padding: var(--s-20) var(--s-12);

  background-color: ${(props) =>
    props.isExit ? "var(--c-orange)" : "var(--c-white)"};
  color: ${(props) => (props.isExit ? "var(--c-white)" : "var(--c-orange)")};

  text-transform: uppercase;
  border-bottom: 1px solid rgba(19, 44, 83, 0.2);
`;
