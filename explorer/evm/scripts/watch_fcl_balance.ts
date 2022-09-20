import { ethers } from 'hardhat';

async function main() {
  const contractAddress = "0x5FbDB2315678afecb367f032d93F642f64180aa3";
  const balanceAddress = "0x2bE695db4A06BbE30DC33fbdc73875ca1F5DeE5D";

  const FCLToken = await ethers.getContractFactory("FCLToken");
  const tokenInstance = await FCLToken.attach(contractAddress);
  await tokenInstance.deployed();

  let lastBalance = null;
  while (true) {
    const balance = await tokenInstance.balanceOf(balanceAddress) / 10**12;

    if (lastBalance == null || (balance - lastBalance) !== 0) {
      console.log(balanceAddress, balance.toString(), "FCL");
      lastBalance = balance;
    }

    await new Promise(resolve => setTimeout(resolve, 5000));
  }
}

main().catch(e => {
  console.error(e);
  process.exit(1);
});
