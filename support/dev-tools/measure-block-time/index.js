const { ApiPromise, WsProvider } = require('@polkadot/api');

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
    const api = await createPromiseApi(nodeAddress, types)

    let count = 0
    let totalBlockTime = 0
    const _rpc = await api.rpc.chain.subscribeNewHeads(async (lastHeader) => {
        const lastBlockNum = lastHeader.number.toNumber()
        const momentCurrent = await api.query.timestamp.now.at(lastHeader.hash)
        const momentPrev = await api.query.timestamp.now.at(lastHeader.parentHash)
        const blockTime = momentCurrent - momentPrev
        console.log("#", lastBlockNum - 1, "-> #", lastBlockNum, ", block time: ", blockTime)

        totalBlockTime += blockTime
        count += 1
        console.log("average block time: ", totalBlockTime / count)
    });
}

main().catch(console.error);
