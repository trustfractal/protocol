import Copy from "@assets/copy.svg";
import Button from "@common/Button";
import {
  BoldText,
  Subsubtitle,
  Text,
  VerticalSequence,
} from "@components/Common";
import { Minting } from "@components/DataScreen/Minting";
import WebpageViews from "@components/DataScreen/WebpageViews";
import RoutesPaths from "@popup/routes/paths";
import { getProtocolOptIn } from "@services/Factory";
import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import styled from "styled-components"

const AddressContainer = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
`;

const LineWithCopy = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;

  > *:not(:last-child) {
    margin-right: 8px;
  }

  > svg {
    &:hover {
      cursor: pointer;
    }
  }
`;

interface AddressProps {
    address: string;
}

function Address({ address }: AddressProps) {
  return (
    <AddressContainer>
      <BoldText>Your Address</BoldText>

      <LineWithCopy>
        <Subsubtitle>{address}</Subsubtitle>

        <Copy onClick={() => navigator.clipboard.writeText(address)} />
      </LineWithCopy>
    </AddressContainer>
  );
}

const LivenessContainer = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
`;

function AddLiveness() {
  const [hasLiveness, setHasLiveness] = useState(true);

  useEffect(() => {
    (async () => {
      setHasLiveness(await getProtocolOptIn().hasCompletedLiveness());
    })();
  });

  if (hasLiveness) return null;

  const postOptInLiveness = async () => {
    await getProtocolOptIn().postOptInLiveness();
  };

  return (
    <LivenessContainer>
      <Text>Add liveness to unlock minting rewards:</Text>
      <Button onClick={postOptInLiveness}>Add Liveness</Button>
    </LivenessContainer>
  );
}

function DataScreen() {
  const [address, setAddress] = useState<string>();
  const navigate = useNavigate();

  useEffect(() => {
    (() => {
      const address = getProtocolOptIn().getAddress();
      if (address) setAddress(address);
    })();
  }, []);

  if (!address) return <></>;

  return (
    <VerticalSequence>
      <AddLiveness />
      <Minting />
      <WebpageViews />
      <Address address={address} />
      <Button
        onClick={() =>
            navigate(RoutesPaths.SEND)
        }
      >
        Send FCL
      </Button>
    </VerticalSequence>
  );
}

DataScreen.defaultProps = {};

export default DataScreen;
