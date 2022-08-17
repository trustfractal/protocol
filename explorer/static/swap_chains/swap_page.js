import { React, ReactDOM, html } from "/static/deps.js";
import { useLoaded, fetchJson, Loading } from "/static/utils.js";

import QRCode from "https://unpkg.com/qrcode.react@3.0.1/lib/esm/index.js";

const Swap = (props) => {
  const id = props.swapId;

  const [swap, setSwap] = React.useState(null);

  React.useEffect(() => {
    let terminate = false;

    (async () => {
      while (!terminate) {
        try {
          const swap = await fetchJson(`/swap_chains/${id}.json`);
          setSwap(swap);
          if (swap.isFinished) break;
        } catch (e) {
          console.error(e);
        }

        await new Promise(resolve => setTimeout(resolve, 5000));
      }
    })();

    return () => {
      terminate = true;
    };
  }, [id]);

  if (swap == null) return Loading();
  console.log('swap', swap);

  let currentState;
  if (swap.state.awaitingReceive !== undefined) {
    const state = swap.state.awaitingReceive;
    currentState = html`
      <div>
        <h2>Awaiting Receive</h2>

        <p>
          Send any amount to <${CopyToClipboard} text=${state.receiveAddress} />
        </p>

        <${QRCode} value=${state.paymentRequest} />
      </div>
    `;
  } else {
    throw new Error(`Unrecognized state ${JSON.stringify(swap.state)}`);
  }

  return html`
    <div>
      <h1>Swap: ${swap.id}</h1>

      ${currentState}

      <pre>${JSON.stringify(swap, null, 2)}</pre>
    </div>
  `;
}

const CopyToClipboard = (props) => {
  const doCopy = async () => {
    await navigator.clipboard.writeText(props.text);
  };

  return html`
    <span className="interactive-text">
      ${props.text}
      <button className="btn" onClick=${doCopy}>
        <i className="material-icons">content_copy</i>
      </button>
    </span>
  `;
};

const pathParts = window.location.pathname.split("/");
const swapId = pathParts[pathParts.length - 1];

ReactDOM.render(
  html`<${Swap} swapId=${swapId} />`,
  document.getElementById("app")
);
