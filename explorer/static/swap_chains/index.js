import { React, ReactDOM, html } from "/static/deps.js";
import { useLoaded, fetchJson, Loading } from "/static/utils.js";

const Index = (props) => {
  const [systemReceive, setSystemReceive] = React.useState(null);
  const [systemSend, setSystemSend] = React.useState(null);
  const [sendAddress, setSendAddress] = React.useState("");
  const [startingSwap, setStartingSwap] = React.useState(false);

  let chainOptions = useLoaded(() => fetchJson("/swap_chains/chain_options.json"), []);
  if (!chainOptions.loaded) return Loading();
  chainOptions = chainOptions.value;

  const receiveButtons = chainOptions.systemReceive.map(chain => {
    return html`
      <button
          className=${chain == systemReceive ? "btn" : "btn-flat"}
          key=${chain.id}
          onClick=${() => setSystemReceive(chain)}>
        ${chain.name}
      </button>
    `;
  });

  const startEnabled = sendAddress != "" && !startingSwap;

  const startSwap = async () => {
    setStartingSwap(true);
    await new Promise(resolve => setTimeout(resolve, 1000));
    setStartingSwap(false);
    // TODO(shelbyd): Pick up here.
    throw new Error('Not implemented!');
  };

  const sendButtons = chainOptions.systemSend.map(chain => {
    return html`
      <button
          className=${chain == systemSend ? "btn" : "btn-flat"}
          key=${chain.id}
          onClick=${() => setSystemSend(chain)}>
        ${chain.name}
      </button>
    `;
  });

  return html`
    <h2>You Send${systemReceive != null && `: ${systemReceive.name}`}</h2>

    <div className="receive-buttons">
      ${receiveButtons}
    </div>

    ${systemReceive != null && html`
      <h2>You Receive${systemSend != null && `: ${systemSend.name}`}</h2>

      <div className="send-buttons">
        ${sendButtons}
      </div>
    `}

    ${systemSend != null && html`
      <div>
        <label>
          <input type="text"
              placeholder="Address"
              value=${sendAddress}
              onChange=${(event) => setSendAddress(event.target.value)} />
          Receive Address
        </label>
      </div>
    `}

    <button
        className=${`btn ${startEnabled ? "" : "disabled"}`}
        onClick=${() => startSwap()}>
      Start
      ${startingSwap && html`<i class="material-icons right">cloud_sync</i>`}
    </button>
  `;
}

ReactDOM.render(
  html`<${Index} />`,
  document.getElementById("app")
);
