import { React, html } from "/static/deps.js";

export async function fetchJson(path, body, opts) {
  const bodyString = typeof body == "object" ? JSON.stringify(body) : body;

  const response = await fetch(path, {
    body: bodyString,
    ...opts,
  });

  if (!response.ok) {
    throw new Error(`Fetch to ${path} failed with status ${response.status}`)
  }
  return await response.json();
}

export function useLoaded(fn, deps) {
  const [result, setResult] = React.useState({loaded: false, value: null});

  React.useEffect(() => {
    (async () => {
      setResult({
        loaded: true,
        value: await fn(),
      });
    })();
  }, deps);

  return result;
}

export const Loading = () => {
  return html`
    <div className="progress">
      <div className="indeterminate"></div>
    </div>
  `;
};
