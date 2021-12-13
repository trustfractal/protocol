import { Environment } from '@popup/types/Environment';

//TODO(melatron): Expose these to clients as configuration with production values being the default.
const FRACTAL_WEBSITE_URL = 'https://staging.sandbox.fractal.id';
const PROTOCOL_RPC_ENDPOINT = 'wss://nodes.testnet.fractalprotocol.com';
const LIVELNESS_CHECK_URL = `https://staging.sandbox.fractal.id/protocol?substrate_address=`

const PROTOCOL_CURRENCY = 'FCL';
const PROTOCOL_JOURNEY_URL = 'https://staging.sandbox.fractal.id';
const NODE_ENV = 'development';

const environment: Environment = {
  FRACTAL_WEBSITE_URL,
  IS_DEV: NODE_ENV === 'development',
  PROTOCOL_RPC_ENDPOINT,
  PROTOCOL_CURRENCY,
  PROTOCOL_JOURNEY_URL,
  LIVELNESS_CHECK_URL,
};

function changeEnviroment(value: Environment) {
  environment.FRACTAL_WEBSITE_URL =
    value.FRACTAL_WEBSITE_URL ?? environment.FRACTAL_WEBSITE_URL;
  environment.IS_DEV = value.IS_DEV ?? environment.IS_DEV;
  environment.PROTOCOL_RPC_ENDPOINT =
    value.PROTOCOL_RPC_ENDPOINT ?? environment.PROTOCOL_RPC_ENDPOINT;
  environment.PROTOCOL_CURRENCY =
    value.PROTOCOL_CURRENCY ?? environment.PROTOCOL_CURRENCY;
}

export { changeEnviroment, environment };
