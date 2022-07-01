const {ApiPromise, WsProvider, Keyring} = require('@polkadot/api');
const {TxnWatcher} = require('@trustfractal/polkadot-utils')

var argv = require('minimist')(process.argv.slice(2));
const fs = require('fs');

// Create a promise API instance of the passed in node address.
async function createPromiseApi(nodeAddress) {
  const wsProvider = new WsProvider(nodeAddress);
  const api = await new ApiPromise({provider : wsProvider});
  await api.isReady;
  return api;
}

async function main() {
  const nodeAddress = getRequiredArg('nodeAddress');
  const api = await createPromiseApi(nodeAddress);
  const keyring = new Keyring({type : 'sr25519'});

  // Some mnemonic phrase
  const PHRASE =
      fs.readFileSync(getRequiredArg('privateKey')).toString().trimEnd();

  // Add an account, straight mnemonic
  const sudoPair = keyring.addFromUri(PHRASE);

  // Retrieve the upgrade key from the chain state
  const adminId = await api.query.sudo.key();
  if (adminId.toString() != sudoPair.address) {
    throw new Error(`The provided key: ${
        sudoPair.address}, doesn't match the sudo key: ${adminId}`);
  }

  // Retrieve the runtime to upgrade
  const code = fs.readFileSync(getRequiredArg('wasmFile')).toString('hex');
  const proposal = api.tx.system.setCode(`0x${code}`);

  console.log(`Upgrading using ${adminId}; ${code.length / 2} bytes`);

  // Perform the actual chain upgrade via the sudo module
  const rawTxn = api.tx.sudo.sudoUncheckedWeight(proposal, 1);
  const txn = TxnWatcher.signAndSend(rawTxn, sudoPair);

  const inBlock = await txn.inBlock();
  console.log(`Included in block: ${inBlock.block}`);

  const finalized = await txn.finalized();
  console.log(`Finalized and included in block: ${finalized.includedInBlock}`);
}

function getRequiredArg(name) {
  if (argv[name] != null) {
    return argv[name];
  }

  throw new Error(`Missing required argument '${name}'`);
}

main().then(() => { process.exit(0); }).catch((error) => {
  console.error(error);
  process.exit(1);
});
