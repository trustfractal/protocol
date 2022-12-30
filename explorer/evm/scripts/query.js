const fs = require("fs");
const { ethers } = require("hardhat");

function writeCSV(filename, rows) {
  return fs.writeFileSync(filename, rows.join("\n"));
}

async function main() {
  const FCLToken = await ethers.getContractFactory("FCLToken");
  const tokenInstance = FCLToken.attach("0xe68856eb29b2fb39699286cca7f10f90ce8ae9de");

  const mintsFilter = tokenInstance.filters.Transfer(ethers.constants.AddressZero, null);
  //const burnsFilter = tokenInstance.filters.Transfer(null, ethers.constants.AddressZero);

  const mints = await tokenInstance.queryFilter(mintsFilter);
  //const burns = await tokenInstance.queryFilter(burnsFilter);

  writeCSV("mints.csv", mints.map(mint => [
    mint.blockNumber,
    mint.transactionHash,
    mint.args.to,
    ethers.utils.formatEther(mint.args.value),
  ]));

  //writeCSV("burns.csv", burns.map(burn => [
  //  burn.blockNumber,
  //  burn.transactionHash,
  //  burn.args.from,
  //  ethers.utils.formatEther(burn.args.value),
  //]));
}

main().catch(e => {
  console.error(e);
  process.exit(1);
});
