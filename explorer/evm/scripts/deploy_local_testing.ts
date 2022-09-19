import { ethers } from 'hardhat';

async function main() {
  const FCLToken = await ethers.getContractFactory("FCLToken");
  const tokenInstance = await FCLToken.deploy();
  await tokenInstance.deployed();

  const minterRole = await tokenInstance.MINTER_ROLE();
  const minter = new ethers.Wallet("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d");
  await tokenInstance.grantRole(minterRole, minter.address);

  console.log(JSON.stringify({
    contract: tokenInstance.address,
    minterAddress: minter.address,
    minterKey: minter.privateKey,
  }, null, 2));
}

main().catch(e => {
  console.error(e);
  process.exit(1);
});
