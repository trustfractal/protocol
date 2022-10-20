import { React, ReactDOM, html, ethers } from "/static/deps.js";
import { useLoaded, fetchJson, Loading } from "/static/utils.js";

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
  if (swap.state.awaitingReceive?.simple !== undefined) {
    currentState = html`<${AwaitingReceive} state=${swap.state.awaitingReceive.simple} user=${swap.user} />`;
  } else if (swap.state.awaitingReceive?.metamask !== undefined) {
    currentState = html`<${AwaitingMetamaskReceive} state=${swap.state.awaitingReceive.metamask} user=${swap.user} />`;
  } else if (swap.state.finalizing !== undefined) {
    currentState = html`<${Finalizing} state=${swap.state.finalizing} user=${swap.user} />`;
  } else if (swap.state.sending !== undefined) {
    currentState = html`<${Sending} state=${swap.state.sending} user=${swap.user} />`;
  } else if (swap.state.finished !== undefined) {
    currentState = html`<${Finished} state=${swap.state.finished} />`;
  } else {
    throw new Error(`Unrecognized state ${JSON.stringify(swap.state)}`);
  }

  return html`
    <div>
      ${currentState}

      ${showJson && html`<pre>${JSON.stringify(swap, null, 2)}</pre>`}
    </div>
  `;
}

const AwaitingReceive = (props) => {
  return html`
    <div>
      <h1>Your swap is prepared</h1>

      <h2>Awaiting your token transfer in <span className="style--capitalize">${props.user.systemReceive}</span>...</h2>

      <${LoadingSpinner} />

      <div className="instructions style--center-text">
        <p>
          To continue, send any FCL amount to
          <br />
          <${CopyToClipboard} text=${props.state.receiveAddress} />
        </p>
      </div>
    </div>
  `;
};

const AwaitingMetamaskReceive = (props) => {
  const [phaseComponent, setPhaseComponent] = React.useState(null);

  const ui = {
    getAmountString: () => new Promise(resolve => {
      setPhaseComponent(html`<${AmountString} onSubmit=${(amount) => resolve(amount)} />`);
    }),
    showMessage: (message, options) => {
      setPhaseComponent(html`
        ${options?.loading && html`<${LoadingSpinner} />` }
        <p className="style--center-text">${message}</p>
      `);
    },
    awaitContinue: () => new Promise(resolve => {
      setPhaseComponent(html`
        <p className="style--center-text">
          <button className="btn" onClick=${() => resolve()}>
            Continue
            <i className="material-icons right">arrow_forward</i>
          </button>
        </p>
      `);
    }),
  };

  React.useEffect(() => {
    (async () => {
      while (true) {
        try {
          await sendMetamaskTransactions(props.state, ui);
          break;
        } catch (e) {
          console.error(e);
        }
      }
    })();
  }, []);

  return html`
    <div className="flex-col">
      <h1>Your swap is prepared</h1>

      <h2>Awaiting your token transfer in <span className="style--capitalize">${props.user?.systemReceive}</span>...</h2>

      ${phaseComponent}
    </div>
  `;
};

async function sendMetamaskTransactions(metamask, ui) {
  const provider = new ethers.providers.Web3Provider(window.ethereum);

  const amountString = await ui.getAmountString();
  const amount = ethers.utils.parseUnits(amountString, metamask.ercDecimals);
  ui.showMessage(`Preparing to send ${amountString} FCL`);

  const chainIdHex = await provider.send('eth_chainId');
  const chainIdNumber = parseInt(chainIdHex.slice(2), 16);
  if (chainIdNumber !== metamask.chainId) {
    await provider.send(
      'wallet_switchEthereumChain',
      [{ chainId: `0x${metamask.chainId.toString(16)}` }],
    );
  }

  let accounts = await provider.send('eth_accounts');
  while (accounts.length === 0) {
    ui.showMessage('Please select a MetaMask account to send from', { loading: true });
    accounts = await provider.send('eth_requestAccounts');
  }

  let txnNumber = 0;
  const signer = provider.getSigner();
  for (const txn of metamask.transactions) {
    const contract = new ethers.Contract(txn.contractAddress, txn.contractAbi, provider);
    const withSigner = contract.connect(signer);

    const params = txn.params.map(p => {
      if (p === "user_amount") {
        return amount;
      } else {
        return p;
      }
    });

    txnNumber += 1;
    ui.showMessage(`(${txnNumber} / ${metamask.transactions.length}) Preparing ${metamask.transactions[txnNumber-1].method} transaction ...`, { loading: true });
    const sentTxn = await withSigner[txn.method](...params);
    ui.showMessage(`(${txnNumber} / ${metamask.transactions.length}) Waiting for ${metamask.transactions[txnNumber-1].method} transaction to be mined...`, { loading: true });
    const waited = await sentTxn.wait();

    const remaining = metamask.transactions.length - txnNumber;
    if (remaining > 0) {
      await ui.awaitContinue(`Transaction in block, ${remaining} remaining...`)
    }
  }

  ui.showMessage('All transactions mined!');
}

const AmountString = (props) => {
  const [amountStr, setAmountStr] = React.useState("");

  const enabled = !!amountStr;

  return html`
    <div className="flex-col">
      <label>Amount of FCL to bridge (minimum 1 FCL):</label>
      <div className="input-field style--no-top-margin">
        <input type="number"
          autoFocus
          min="1"
          step="0.1"
          required
          onChange=${(event) => setAmountStr(event.target.validity.valid ? event.target.value : "")} />
      </div>

      <button className="btn" disabled=${!enabled} onClick=${() => props.onSubmit(amountStr)}>
        Send in MetaMask
        <i className="material-icons right">open_in_new</i>
      </button>
    </div>
  `;
};

const CopyToClipboard = (props) => {
  const doCopy = async () => {
    await navigator.clipboard.writeText(props.text);
  };

  return html`
    <span className="interactive-text">
      <span className="style--monospace style--wrap-text">${props.text}</span>
      <button className="btn-flat" onClick=${doCopy}>
        <i className="material-icons">content_copy</i>
      </button>
    </span>
  `;
};

const Finalizing = (props) => {
  return html`
    <div>
      <h1>Your swap is ongoing</h1>

      <h2>Burning your FCL in <span className="style--capitalize">${props.user.systemReceive}</span>...</h2>

      <${LoadingSpinner} />
    </div>
  `;
};

const Sending = (props) => {
  return html`
    <div>
      <h1>Your swap is ongoing</h1>

      <h2>Minting your FCL in <span className="style--capitalize">${props.user.systemSend}</span>...</h2>

      <${LoadingSpinner} />
    </div>
  `;
};

const LoadingSpinner = (props) => {
  return html`
    <div className="spinner">
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
    </div>
  `;
};

const Finished = (props) => {
  return html`
    <div>
      <h1>Your swap is complete</h1>

      <img className="flavour-img" src="/static/swap_chains/swap_done.svg" />

      <p className="style--center-text">
        Transaction: <a href=${props.state.txnLink} target="_blank">${props.state.txnId}</a>
      </p>

      <p className="style--center-text">
        <a className="btn" href="/swap_chains">
          <i className="material-icons left">cloud_sync</i>
          New swap
        </a>
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
