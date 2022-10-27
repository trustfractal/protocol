const { ethers } = require('hardhat');
const { txParams } = require("./utils/karuraAndAcalaTransactionHelpers");

const { gasPrice, gasLimit } = await txParams();

const [signer] = await ethers.getSigners();

const FCLToken = await ethers.getContractFactory("FCLToken");
const tokenInstance = await FCLToken.deploy({ gasPrice, gasLimit });
await tokenInstance.deployed();
//const tokenInstance = FCLToken.attach("0xe233eD7e1bb733647fD1b10F28F12e30781589C6");

const minterRole = await tokenInstance.MINTER_ROLE();
const minter = new ethers.Wallet("PRIVATE KEY");
await tokenInstance.grantRole(minterRole, minter.address, { gasPrice, gasLimit });

const FCLBurner = await ethers.getContractFactory("FCLBurner");
const burnerInstance = await FCLBurner.deploy(tokenInstance.address, { gasPrice, gasLimit });
await burnerInstance.deployed();

console.log(JSON.stringify({
  tokenContract: tokenInstance.address,
  minterAddress: minter.address,
  minterKey: minter.privateKey,
  burnerContract: burnerInstance.address,
}, null, 2));


await tokenInstance.mint(signer.address, 100, { gasPrice, gasLimit });
await (await tokenInstance.approve(burnerInstance.address, 10, { gasPrice, gasLimit })).wait(1)
await tokenInstance.allowance(signer.address, burnerInstance.address)
await burnerInstance.burn("derp", 9, { gasPrice, gasLimit });

