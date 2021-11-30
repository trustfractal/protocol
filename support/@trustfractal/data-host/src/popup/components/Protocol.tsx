import TopComponent from '@common/TopComponent';
import Loading from "@components/Loading";
import { OptInForm } from "@components/OptInForm";
import { SetupInProgress, SetupSuccess } from "@components/SetupScreen";
import { mnemonicGenerate } from "@polkadot/util-crypto";
import { getProtocolOptIn } from "@services/Factory";
import { useLoadedState } from "@utils/ReactHooks";
import { useState } from "react";

function DataScreen() {
    return <div></div>
}

function ProtocolState() {
    const [pageOverride, setPageOverride] = useState<JSX.Element | null>(null);

    const serviceOptedIn = useLoadedState(() => getProtocolOptIn().isOptedIn());
    const completedLiveness = useLoadedState(() =>
      getProtocolOptIn().hasCompletedLiveness(),
    );

    const optInWithMnemonic = async (mnemonic?: string) => {
      mnemonic = mnemonic || mnemonicGenerate();
      try {
        setPageOverride(
          <SetupInProgress onRetry={() => optInWithMnemonic(mnemonic)} />,
        );

        await getProtocolOptIn().optIn(mnemonic);
        serviceOptedIn.reload();
        completedLiveness.reload();

        setPageOverride(
          <SetupSuccess
            mnemonic={mnemonic}
            onContinue={() => setPageOverride(null)}
          />,
        );
      } catch (e) {
        console.log(e);
        // handleError(e, () => optInWithMnemonic(mnemonic));
      }
    };

    if (pageOverride != null) {
        return pageOverride;
      }

    if (!serviceOptedIn.isLoaded) return <Loading />;
    if (!serviceOptedIn.value) {
      return <OptInForm onOptIn={() => optInWithMnemonic()} />;
    }

    return <DataScreen />;
  }

function Protocol() {
    return (
      <TopComponent>
          <ProtocolState />
      </TopComponent>
    );
  }

  export default Protocol;
