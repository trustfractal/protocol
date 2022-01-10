import Menu from "@common/Menu";
import type { AccountData, Balance } from "@polkadot/types/interfaces";
import { environment } from '@popup/Environment';
import { IconNames } from "@popup/components/common/Icon";
import Logo, { LogoSizes } from "@popup/components/common/Logo";
import Text, {
  TextHeights,
  TextSizes,
  TextWeights,
} from "@popup/components/common/Text";
import RoutesPaths from "@popup/routes/paths";
import {
  getProtocolOptIn,
  getProtocolService,
  getUserAlerts,
} from "@services/Factory";
import { formatBalance } from "@utils/FormatUtils";
import { useLoadedState } from "@utils/ReactHooks";
import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import styled from "styled-components";

const NavbarContainer = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;

  padding: var(--s-19) var(--s-24);

  border-bottom: 1px solid var(--c-orange);
`;

const HeaderContainer = styled.div`
  display: flex;
  flex-direction: row;
  justify-content: flex-start;
  align-items: center;
`;

const LogoContainer = styled.div`
  display: flex;
  justify-content: flex-start;

  margin-right: var(--s-24);
`;

const RootContainer = styled.div`
  position: relative;
  overflow: hidden;

  min-width: 400px;
  min-height: 460px;
`;

const BalanceContainer = styled.div`
  display: flex;
  flex-direction: column;
  width: 100%;
`;

const BalanceTitleContainer = styled.div`
  color: var(--c-orange);
`;

const BalanceTextContainer = styled.div`
  display: flex;
  flex-direction: row;
  width: 100%;
  align-items: flex-end;
`;

const BalanceFreeContainer = styled.div`
  display: flex;
  flex-direction: row;
  align-items: flex-end;
  margin-right: var(--s-10);
`;

const BalanceReservedContainer = styled.span`
  display: flex;
  flex-direction: row;
  align-items: flex-end;
`;

const BalanceFree = styled.span`
  margin-right: var(--s-3);
`;

const BalanceReserved = styled.span<{ isPositive: boolean }>`
  color: ${(props) => (props.isPositive ? "var(--c-green)" : "var(--c-red)")};
  margin-right: var(--s-3);
`;

const BalanceReservedLabel = styled.span`
  opacity: 0.6;
`;

function DropdownMenu() {
  const navigate = useNavigate();
  const mnemonic = useLoadedState(() => getProtocolOptIn().getMnemonic());

  const menuItems = [
    {
      label: "Backup protocol wallet",
      icon: IconNames.IMPORT,
      onClick: async () => {
        await navigator.clipboard.writeText(mnemonic.unwrapOrDefault("")!);
        getUserAlerts().send("Mnemonic copied to clipboard!");
      },
      disabled: !mnemonic.isLoaded || mnemonic.value == null,
    },
    {
      label: "Sweep funds",
      icon: IconNames.FRACTAL_TOKEN,
      onClick: () => {
        navigate(RoutesPaths.SWEEP)
      },
      disabled: !mnemonic.isLoaded || mnemonic.value == null,
    },
    {
      label: "About",
      icon: IconNames.ABOUT,
      onClick: () => {
        navigate(RoutesPaths.ABOUT)
      },
    },
  ];

  if (environment.IS_DEV) {
       //TODO(melatron): Remove
    // menuItems.push({
    //   label: "Clear Tokens",
    //   icon: IconNames.INVALID,
    //   onClick: () => getFractalAccountConnector().clearTokens(),
    // });
  }

  return <Menu items={menuItems} />;
}

const toHuman = (balance: Balance) => Number(balance.toBigInt()) / 10 ** 12;

function ProtocolReservedBalance({ reserved }: { reserved: Balance }) {
  const reservedHuman = toHuman(reserved);

  if (reservedHuman === 0) return <></>;

  const isPositive = reservedHuman > 0;

  return (
    <BalanceReservedContainer>
      <BalanceReserved isPositive={isPositive}>
        <Text
          size={TextSizes.MEDIUM}
          height={TextHeights.MEDIUM}
          weight={TextWeights.BOLD}
        >
          {isPositive ? "+" : "-"}
          {formatBalance(reservedHuman)}
        </Text>
      </BalanceReserved>

      <BalanceReservedLabel>
        <Text size={TextSizes.SMALL} height={TextHeights.SMALL}>
          EXPECTED INCREASE
        </Text>
      </BalanceReservedLabel>
    </BalanceReservedContainer>
  );
}

function ProtocolBalance({ balance }: { balance: AccountData }) {
  const freeHuman = toHuman(balance.free);

  return (
    <BalanceContainer>
      <BalanceTitleContainer>
        <Text
          size={TextSizes.SMALL}
          height={TextHeights.SMALL}
          weight={TextWeights.BOLD}
        >
          BALANCE
        </Text>
      </BalanceTitleContainer>
      <BalanceTextContainer>
        <BalanceFreeContainer>
          <BalanceFree>
            <Text
              size={TextSizes.MEDIUM}
              height={TextHeights.MEDIUM}
              weight={TextWeights.BOLD}
            >
              {formatBalance(freeHuman)}
            </Text>
          </BalanceFree>

          <Text size={TextSizes.SMALL} height={TextHeights.SMALL}>
            {environment.PROTOCOL_CURRENCY}
          </Text>
        </BalanceFreeContainer>
        <ProtocolReservedBalance reserved={balance.reserved} />
      </BalanceTextContainer>
    </BalanceContainer>
  );
}

function BalanceHeader({ balance }: { balance: AccountData }) {
  return (
    <HeaderContainer>
      <LogoContainer>
        <Logo size={LogoSizes.SMALL} />
      </LogoContainer>

      <ProtocolBalance balance={balance} />
    </HeaderContainer>
  );
}

function LogoHeader() {
  return (
    <HeaderContainer>
      <LogoContainer>
        <Logo size={LogoSizes.SMALL} />
      </LogoContainer>
      <Text
        size={TextSizes.LARGE}
        height={TextHeights.LARGE}
        weight={TextWeights.BOLD}
      >
        Fractal Wallet
      </Text>
    </HeaderContainer>
  );
}

function Header() {
  const [balance, setBalance] = useState<AccountData>();

  useEffect(() => {
    let unsub: () => void;
    (async () => {
      unsub = await getProtocolService().watchBalance(setBalance);
    })();
    return () => {
      if (unsub) unsub();
    };
  }, []);

  if (balance) {
    return <BalanceHeader balance={balance} />;
  }

  return <LogoHeader />;
}

function Navbar() {
  return (
    <NavbarContainer>
      <Header />
      <DropdownMenu />
    </NavbarContainer>
  );
}

const withNavBar =
  <P extends object>(
    Component: React.ComponentType<P>,
    withNavBarComponent = true,
  ) =>
  (props: any) => {
    const ref = React.createRef<HTMLDivElement>();

    return (
      <>
        <RootContainer ref={ref}>
          {withNavBarComponent && <Navbar />}
          <Component {...(props as P)} />
        </RootContainer>
      </>
    );
  };

  export default withNavBar
