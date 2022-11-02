const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { TxnWatcher } = require('@trustfractal/polkadot-utils')

var argv = require('minimist')(process.argv.slice(2));
const fs = require('fs');

// Create a promise API instance of the passed in node address.
async function createPromiseApi(nodeAddress) {
    const wsProvider = new WsProvider(nodeAddress);
    const api = await new ApiPromise({ provider: wsProvider });
    await api.isReady;
    return api;
}

async function main() {
    const nodeAddress = argv.nodeAddress;
    const api = await createPromiseApi(nodeAddress);
    const keyring = new Keyring({ type: 'sr25519' });

    // Some mnemonic phrase
    const PHRASE = fs.readFileSync(argv.privateKey).toString().trimEnd();

    // Add an account, straight mnemonic
    const newPair = keyring.addFromUri(PHRASE);

    // Retrieve the runtime to upgrade
    const code = fs.readFileSync(argv.wasmPath).toString('hex');
    const proposal = api.tx.system.setCode(`0x${code}`);

    // Retrieve the upgrade key from the chain state
    const adminId = await api.query.sudo.key();

    if (`${adminId}` != newPair.address) {
        console.error(`The provided key - ${newPair.address} doesn't match the sudo key - ${adminId}`)
        process.exit(0);
    }
    console.log(`Upgrading from ${adminId}, ${code.length / 2} bytes`);

    // Perform the actual chain upgrade via the sudo module
    const txn = TxnWatcher.signAndSend(api.tx.sudo.sudoUncheckedWeight(proposal, 0), newPair);
    let data = await txn.inBlock();
    console.log('Included in block'); //TODO: console.log(`Included in block - ${data.block}`);

    let finalizedData = await txn.finalized();
    console.log('Finalized'); //TODO console.log(`Finalized and included in block - ${finalizedData.includedInBlock}`);
    process.exit(0);
}

main().catch((error) => {
    console.error(error);
    process.exit(1);
  });
