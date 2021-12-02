import TopComponent from '@common/TopComponent';
import Loading from "@components/Loading";
import { OptInForm } from "@components/OptInForm";
import { SetupError, SetupInProgress, SetupSuccess } from "@components/SetupScreen";
import { mnemonicGenerate } from "@polkadot/util-crypto";
import { getProtocolOptIn } from "@services/Factory";
import { useLoadedState } from "@utils/ReactHooks";
import { useState } from "react";

// TODO(melatron): Implement DataScreen
function DataScreen() {
    return <div>Data Screen</div>
}

function ProtocolState() {
    const [pageOverride, setPageOverride] = useState<JSX.Element | null>(null);

    const serviceOptedIn = useLoadedState(() => getProtocolOptIn().isOptedIn());
    const completedLiveness = useLoadedState(() =>
      getProtocolOptIn().hasCompletedLiveness(),
    );

    const handleError = (err: Error, retry: () => void) => {
      console.error(err);
      setPageOverride(<SetupError onRetry={retry} />);
    };

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
      } catch (e: any) {
        handleError(e, () => optInWithMnemonic(mnemonic));
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
