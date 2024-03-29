const hardhat = require("hardhat");
const { calcEthereumTransactionParams } = require("@acala-network/eth-providers");

async function txParams() {
  let storageByteDeposit;

  if (hardhat.network.name === "mandala" || hardhat.network.name === "karura") {
    storageByteDeposit = "100000000000000";
  } else if (hardhat.network.name === "acala" || hardhat.network.name === "acalaBeta") {
    storageByteDeposit = "300000000000000";
  } else {
    throw new Error(`Unconfigured storageByteDeposit for network ${hardhat.network.name}`);
  }
  const txFeePerGas = "199999946752";
  const blockNumber = await ethers.provider.getBlockNumber();

  const ethParams = calcEthereumTransactionParams({
    gasLimit: "31000000",
    validUntil: (blockNumber + 100).toString(),
    storageLimit: "64001",
    txFeePerGas,
    storageByteDeposit,
  });

  return {
    gasPrice: ethParams.txGasPrice,
    gasLimit: ethParams.txGasLimit,
  };
}

module.exports = { txParams };
