import { React, ReactDOM, html } from "/static/deps.js";
import { useLoaded, fetchJson, Loading } from "/static/utils.js";

import QRCode from "https://unpkg.com/qrcode.react@3.0.1/lib/esm/index.js";

const Swap = (props) => {
  const id = props.swapId;

  const [swap, setSwap] = React.useState(null);
  const [showJson, setShowJson] = React.useState(false);

  React.useEffect(() => {
    let terminate = false;

    (async () => {
      while (!terminate) {
        try {
          const swap = await fetchJson(`/swap_chains/${id}.json`);
          setSwap(swap);
          if (swap.state.finished !== undefined) break;
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

  React.useEffect(() => {
    const onKeyPress = (event) => {
      const isJsonToggle = event.key === '?' && event.ctrlKey && event.shiftKey;
      if (!isJsonToggle) return;

      setShowJson(v => !v);
    };

    document.body.addEventListener("keydown", onKeyPress);
    return () => {
      document.body.removeEventListener("keydown", onKeyPress);
    }
  }, []);

  if (swap == null) return Loading();

  let currentState;
  if (swap.state.awaitingReceive !== undefined) {
    currentState = html`<${AwaitingReceive} state=${swap.state.awaitingReceive} />`;
  } else if (swap.state.finalizing !== undefined) {
    currentState = html`<${Finalizing} state=${swap.state.finalizing} />`;
  } else if (swap.state.finished !== undefined) {
    currentState = html`<${Finished} state=${swap.state.finished} />`;
  } else {
    throw new Error(`Unrecognized state ${JSON.stringify(swap.state)}`);
  }

  return html`
    <div>
      <h1>Swap: ${swap.id}</h1>

      ${currentState}

      ${showJson && html`<pre>${JSON.stringify(swap, null, 2)}</pre>`}
    </div>
  `;
}

const AwaitingReceive = (props) => {
  return html`
    <div>
      <h2>Awaiting Receive</h2>

      <p>
        Send any amount to <${CopyToClipboard} text=${props.state.receiveAddress} />
      </p>

      <${QRCode} value=${props.state.paymentRequest} />
    </div>
  `;
};

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

const Finalizing = (props) => {
  return html`
    <div>
      <h2>Finalizing</h2>

      <p>
        Transaction received, waiting for finalization.
      </p>

      <${LoadingSpinner} />
    </div>
  `;
};

const LoadingSpinner = (props) => {
  return html`
    <div className="preloader-wrapper big active">
      <div className="spinner-layer spinner-blue-only">
        <div className="circle-clipper left">
          <div className="circle"></div>
        </div><div className="gap-patch">
          <div className="circle"></div>
        </div><div className="circle-clipper right">
          <div className="circle"></div>
        </div>
      </div>
    </div>
  `;
};

const Finished = (props) => {
  return html`
    <div>
      <h2>Swap Finished</h2>

      <p>
        Transaction: <a href=${props.state.txnLink}>${props.state.txnId}</a>
      </p>
    </div>
  `;
};

const pathParts = window.location.pathname.split("/");
const swapId = pathParts[pathParts.length - 1];

ReactDOM.render(
  html`<${Swap} swapId=${swapId} />`,
  document.getElementById("app")
);
