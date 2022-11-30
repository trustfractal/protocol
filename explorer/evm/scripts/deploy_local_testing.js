const { ethers } = require('hardhat');

async function main() {
  const FCLToken = await ethers.getContractFactory("FCLToken");
  const tokenInstance = await FCLToken.deploy();
  await tokenInstance.deployed();

  const minterRole = await tokenInstance.MINTER_ROLE();
  const minter = new ethers.Wallet("SOME_PRIVATE_KEY");
  await tokenInstance.grantRole(minterRole, minter.address);

  const FCLBurner = await ethers.getContractFactory("FCLBurner");
  const burnerInstance = await FCLBurner.deploy(tokenInstance.address);
  await burnerInstance.deployed();

  console.log(JSON.stringify({
    tokenContract: tokenInstance.address,
    minterAddress: minter.address,
    minterKey: minter.privateKey,
    burnerContract: burnerInstance.address,
  }, null, 2));
}

main().catch(e => {
  console.error(e);
  process.exit(1);
});
