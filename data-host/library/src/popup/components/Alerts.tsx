import styled from "styled-components";
import { useState, useEffect } from "react";
import { getUserAlerts } from "@services/Factory";

export class UserAlerts {
  public onMessage = (_: string) => {};

  send(message: string) {
    this.onMessage(message);
  }
}

const AlertContainer = styled.div`
  position: relative;
`;

const AlertMessage = styled.div`
  position: absolute;
  top: 0;
  left: 0;
  right: 0;

  padding: 8px;
  background-color: var(--c-orange);
  z-index: 1000;
  box-sizing: border-box;

  display: flex;
  flex-direction: row;
  justify-content: center;
  text-align: center;

  animation-duration: 0.5s;
  animation-name: height-in;

  @keyframes height-in {
    from {
      height: 0%;
      padding: 0;
    }
  }
`;

export function AlertsDisplay() {
  const [message, setMessage] = useState<string>();

  useEffect(() => {
    let latestTimeout: any;

    getUserAlerts().onMessage = (m) => {
      setMessage(m);
      clearTimeout(latestTimeout);
      latestTimeout = setTimeout(() => setMessage(undefined), 10 * 1000);
    };
  }, []);

  if (message) {
    return (
      <AlertContainer>
        <AlertMessage>
          <p>
            <strong>{message}</strong>
          </p>
        </AlertMessage>
      </AlertContainer>
    );
  } else {
    return <></>;
  }
}
