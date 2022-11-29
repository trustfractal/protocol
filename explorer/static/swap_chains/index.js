import { React, ReactDOM, html } from "/static/deps.js";
import { useLoaded, fetchJson, Loading } from "/static/utils.js";

const Index = (props) => {
  const [systemReceive, setSystemReceive] = React.useState(null);
  const [systemSend, setSystemSend] = React.useState(null);
  const [sendAddress, setSendAddress] = React.useState("");
  const [showTerms, setShowTerms] = React.useState(false);
  const [termsAccepted, setTermsAccepted] = React.useState(false);
  const [startingSwap, setStartingSwap] = React.useState(false);

  let chainOptions = useLoaded(() => fetchJson("/swap_chains/chain_options.json"), []);
  if (!chainOptions.loaded) return Loading();
  chainOptions = chainOptions.value;

  const resetSystemReceive = (id) => {
    setSystemReceive(getReceiveChain(id));
    setSystemSend(null);
    setSendAddress("");
    setShowTerms(false);
  };

  const getReceiveChain = (id) => chainOptions.systemReceive.find((chain) => (chain.id === id));

  const getSendChain = (id) => chainOptions.systemReceive.find((chain) => (chain.id === id));

  const receiveButtons = chainOptions.systemReceive.map(chain => {
    return html`
      <option
          key=${chain.id}
          value=${chain.id}>
        ${chain.name}
      </option>
    `;
  });

  const sendButtons = chainOptions.systemSend
    .filter(chain => systemReceive?.can_bridge_to?.includes(chain.id))
    .map(chain => {
      return html`
        <option
            key=${chain.id}
            value=${chain.id}>
          ${chain.name}
        </option>
      `;
    });

  const startEnabled = sendAddress != "" && termsAccepted === true && !startingSwap;

  const startSwap = async () => {
    try {
      setStartingSwap(true);
      const body = {
        systemReceive: systemReceive.id,
        systemSend: systemSend.id,
        sendAddress: sendAddress,
      };
      const startedId = await fetchJson("/swap_chains/create.json", body, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
      });

      window.location.href = `/swap_chains/${startedId}`;
    } catch (e) {
      // TODO(shelbyd): Handle errors better.
      console.error(e);
      setStartingSwap(false);
    }
  };

  return html`
    <div>
      <h1>Swap FCL Between Chains</h1>

      <img className="flavour-img" src="/static/swap_chains/swap.svg" />

      <label>You will send FCL from:</label>

      <div className="receive-buttons">
        <select required onChange=${(event) => resetSystemReceive(event.target.value)} value="${systemReceive?.id || ""}">
          <option value="" disabled>Choose your option</option>
          ${receiveButtons}
        </select>
      </div>

      ${systemReceive && html`
        <div>
          <label>You want to receive FCL in:</label>

          <div className="send-buttons">
            <select required onChange=${(event) => setSystemSend(getSendChain(event.target.value))} value="${systemSend?.id || ""}">
              <option value="" disabled>Choose your option</option>
              ${sendButtons}
            </select>
          </div>
        </div>
      `}

      ${systemSend != null && html`
        <div className="${systemSend.id}">
          <${ReceiveAddress}
              withChain=${systemSend}
              onChange=${(value, valid) => {setSendAddress(valid ? value : ""); setShowTerms(showTerms || valid);}} />
        </div>
      `}

      ${showTerms === true && html`
        <div>
          <label>You read and agreed to the <a href="/static/swap_chains/end-user-agreement.pdf" target="_blank">User Agreement</a>:</label>

          <div className="accept-terms">
            <label className="style--no-top-margin">
              <input type="checkbox" checked="${termsAccepted}" onChange=${(event) => setTermsAccepted(event.target.checked)} />
              <span>Yes</span>
            </label>
          </div>
        </div>
      `}

      <p className="style--center-text">
        <button
            className=${`btn btn-large ${startEnabled ? "" : "disabled"}`}
            onClick=${() => startSwap()}>
          ${startingSwap ? "Starting..." : "Start"}
        </button>
      </p>
    </div>
  `;
};

const ReceiveAddress = (props) => {
  const [address, setAddress] = React.useState("");
  const [addressValidity, setAddressValidity] = React.useState({});

  const {id: chain, name: chainName } = props.withChain;

  const getValidity = (chain, address) => {
    return addressValidity[chain]?.[address];
  };
  const validity = getValidity(chain, address);

  const onAddressChange = async (address) => {
    setAddress(address);

    let validity = getValidity(chain, address);
    if (validity == null) {
      const response = await fetchJson("/swap_chains/validate_address.json", {
        address,
        chain,
      }, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
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
    <div>
      <label>You will receive FCL in ${chainName} on this address:</label>
      <div className="input-field style--no-top-margin">
        <input type="text"
            id="receive-address"
            className=${validity === true ? "valid" : address !== "" ? "invalid" : ""}
            value=${address}
            onChange=${(event) => onAddressChange(event.target.value)} />
        <span className="helper-text" data-error="Please enter a valid ${chainName} address"></span>
      </div>
    </div>
  `;
};

ReactDOM.render(
  html`<${Index} />`,
  document.getElementById("app")
);
