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
    try {
      setStartingSwap(true);
      const body = {
        systemReceive: systemReceive.id,
        systemSend: systemSend.id,
        sendAddress: sendAddress,
      };
      const startedId = await fetchJson("/swap_chains/create.json", body, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      window.location.href = `/swap_chains/${startedId}`;
    } catch (e) {
      // TODO(shelbyd): Handle errors better.
      console.error(e);
      setStartingSwap(false);
    }
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
    <div>
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
          <${ReceiveAddress}
              withChain=${systemSend.id}
              onChange=${(value, valid) => setSendAddress(valid ? value : '')} />
        </div>
      `}

      <button
          className=${`btn ${startEnabled ? "" : "disabled"}`}
          onClick=${() => startSwap()}>
        Start
        ${startingSwap && html`<i className="material-icons right">cloud_sync</i>`}
      </button>
    </div>
  `;
};

const ReceiveAddress = (props) => {
  const [address, setAddress] = React.useState('');
  const [addressValidity, setAddressValidity] = React.useState({});

  const chain = props.withChain;

  const getValidity = (chain, address) => {
    return addressValidity[chain]?.[address];
  };
  const validity = getValidity(chain, address);

  const icon = (!address || validity === false) ? 'close' :
      validity == null ? 'refresh' :
      'done';


  const onAddressChange = async (address) => {
    setAddress(address);

    let validity = getValidity(chain, address);
    if (validity == null) {
      const response = await fetchJson("/swap_chains/validate_address.json", {
        address,
        chain,
      }, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      setAddressValidity(v => {
        const chainValidity = v[chain] ?? {};
        const withThis = {...chainValidity, [address]: response.valid};
        return {...v, [chain]: withThis};
      });
      validity = response.valid;
    }

    props.onChange(address, validity === true);
  };

  return html`
    <div className="input-field">
      <i className="material-icons prefix">${icon}</i>
      <label htmlFor="receive-address">Receive Address</label>
      <input type="text"
          id="receive-address"
          value=${address}
          onChange=${(event) => onAddressChange(event.target.value)} />
    </div>
  `;
};

ReactDOM.render(
  html`<${Index} />`,
  document.getElementById("app")
);
