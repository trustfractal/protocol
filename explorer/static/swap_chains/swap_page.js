import { React, ReactDOM, html } from "/static/deps.js";
import { useLoaded, fetchJson, Loading } from "/static/utils.js";

const Swap = (props) => {
  const id = props.swapId;

  const [swap, setSwap] = React.useState(null);

  React.useEffect(() => {
    let terminate = false;

    (async () => {
      while (true) {
        const swap = await fetchJson(`/swap_chains/${id}.json`);
        setSwap(swap);
        if (swap.isFinished) break;
        if (terminate) break;

        await new Promise(resolve => setTimeout(resolve, 5000));
      }
    })();

    return () => {
      terminate = true;
    };
  }, [id]);

  if (swap == null) return Loading();

  return html`
    <h1>Swap: ${swap.id}</h1>
    <pre>${JSON.stringify(swap)}</pre>
  `;
}

const pathParts = window.location.pathname.split("/");
const swapId = pathParts[pathParts.length - 1];

ReactDOM.render(
  html`<${Swap} swapId=${swapId} />`,
  document.getElementById("app")
);
