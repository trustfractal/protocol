const { calcEthereumTransactionParams } = require("@acala-network/eth-providers");

async function txParams() {
    const txFeePerGas = '199999946752';
    const storageByteDeposit = '100000000000000';
    const blockNumber = await ethers.provider.getBlockNumber();

    const ethParams = calcEthereumTransactionParams({
      gasLimit: '31000000',
      validUntil: (blockNumber + 100).toString(),
      storageLimit: '64001',
      txFeePerGas,
      storageByteDeposit
    });

    return {
        gasPrice: ethParams.txGasPrice,
        gasLimit: ethParams.txGasLimit
    };
}

module.exports = { txParams };
