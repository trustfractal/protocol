const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require("@polkadot/keyring");

// Create a promise API instance of the passed in node address.
async function createPromiseApi(nodeAddress, types) {
    const provider = new WsProvider(nodeAddress);
    const api = await new ApiPromise({ provider, types });
    await api.isReady;
    return api;
}

async function main() {
    // const nodeAddress = 'wss://rpc.polkadot.io';
    const nodeAddress = 'ws://127.0.0.1:9944'
    const types = {
        FractalId: "u64",
        MerkleTree: "Raw",
    };

    const fractalId = 1;
    const api = await createPromiseApi(nodeAddress, types)
    const keyring = new Keyring({ type: "sr25519" });
    const signer = keyring.createFromUri('//Alice');
    const address = keyring.createFromUri('//Bob').address;
    let txs = [];
    // construct a list of transactions we want to batch
    for (let i = 0; i < 100; i++) {
        // const nonce = await api.rpc.system.accountNextIndex(signer.address);
        txs.push(api.tx.fractalMinting
            .registerIdentity(fractalId, address));
    }
    // construct the batch and send the transactions
    console.log(api.tx.utility);
    let result = await api.tx.utility
        .batch(txs)
        .signAndSend(signer, ({ status }) => {
            if (status.isInBlock) {
                console.log(`included in ${status.asInBlock}`);
            }
        });
    console.log('======================================================')
    console.log(result)
    console.log('======================================================')
    return result;
}

main().catch(console.error);
