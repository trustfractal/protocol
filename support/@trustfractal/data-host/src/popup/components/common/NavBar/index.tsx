import React, { useEffect, useState, useContext } from "react";
import { useHistory } from "react-router";
import styled from "styled-components";
import type { AccountData, Balance } from "@polkadot/types/interfaces";

import { useAppDispatch } from "@redux/stores/application/context";
import appActions from "@redux/stores/application/reducers/app";

import Logo, { LogoSizes } from "@popup/components/common/Logo";
import Text, {
  TextHeights,
  TextSizes,
  TextWeights,
} from "@popup/components/common/Text";
import { IconNames } from "@popup/components/common/Icon";
import Menu from "@popup/components/common/Menu";
import CredentialsCollection from "@models/Credential/CredentialsCollection";
import { ActivityStackContext } from "@popup/containers/ActivityStack";
import { SweepTokens } from "@popup/components/Protocol/SweepTokens";

import { exportFile } from "@utils/FileUtils";
import { useLoadedState, useObservedState } from "@utils/ReactHooks";

import RoutesPaths from "@popup/routes/paths";
import {
  getFractalAccountConnector,
  getProtocolService,
  getProtocolOptIn,
  getUserAlerts,
} from "@services/Factory";
import { credentialsSubject } from "@services/Observables";

import environment from "@environment/index";

import { formatBalance } from "@utils/FormatUtils";

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
  const history = useHistory();
  const credentials = useObservedState(
    () => credentialsSubject,
  ).unwrapOrDefault(CredentialsCollection.empty());

  const onClickExport = async () =>
    exportFile(credentials.serialize(), "fractal_wallet.backup");

  const onClickAbout = () => history.push(RoutesPaths.ABOUT);

  const mnemonic = useLoadedState(() => getProtocolOptIn().getMnemonic());

  const { updater: activityStack } = useContext(ActivityStackContext);

  let menuItems = [
    {
      label: "Export your credentials",
      icon: IconNames.EXPORT,
      onClick: onClickExport,
      disabled: credentials.length === 0,
    },
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
        activityStack.push(
          <SweepTokens onFinish={() => activityStack.pop()} />,
        );
      },
      disabled: !mnemonic.isLoaded || mnemonic.value == null,
    },
    {
      label: "About",
      icon: IconNames.ABOUT,
      onClick: onClickAbout,
    },
  ];

  if (environment.IS_DEV) {
    menuItems.push({
      label: "Clear Tokens",
      icon: IconNames.INVALID,
      onClick: () => getFractalAccountConnector().clearTokens(),
    });
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

export const withNavBar =
  <P extends object>(
    Component: React.ComponentType<P>,
    withNavBarComponent = true,
  ) =>
  (props: any) => {
    const appDispatch = useAppDispatch();

    const ref = React.createRef<HTMLDivElement>();

    useEffect(() => {
      if (ref.current !== null) {
        const element = ref.current;
        const height =
          Math.max(element.scrollHeight, element.offsetHeight) + 24;

        appDispatch(appActions.setPopupSize({ height }));
      }
    }, [ref, appDispatch]);

    return (
      <>
        <RootContainer ref={ref}>
          {withNavBarComponent && <Navbar />}
          <Component {...(props as P)} />
        </RootContainer>
      </>
    );
  };
