import { Environment } from '@popup/types/Environment';

const FRACTAL_WEBSITE_URL =
//   process.env.REACT_APP_FRACTAL_WEBSITE_URL ||
  'https://staging.sandbox.fractal.id';
const PROTOCOL_RPC_ENDPOINT =
//   process.env.REACT_APP_PROTOCOL_RPC_ENDPOINT ||
  'wss://nodes.testnet.fractalprotocol.com';

const PROTOCOL_CURRENCY =
    // process.env.REACT_APP_PROTOCOL_CURRENCY ||
    'FCL';


const NODE_ENV =
    // process.env.NODE_ENV ||
    'development';

const environment: Environment = {
  FRACTAL_WEBSITE_URL,
  IS_DEV: NODE_ENV === 'development',
  PROTOCOL_RPC_ENDPOINT,
  PROTOCOL_CURRENCY,
};

export default environment;
