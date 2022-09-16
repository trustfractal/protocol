const { expect } = require("chai");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

describe("FCLToken", function () {
  async function deployFixture () {
    const [owner, account1] = await ethers.getSigners();

    const FCLToken = await ethers.getContractFactory("FCLToken");

    const fclToken = await FCLToken.deploy();

    const minterRole = await fclToken.MINTER_ROLE();

    return { owner, account1, fclToken, minterRole };
  };

  describe("Minting role", function () {
    it("Should initially belong to deployer", async function () {
      const { owner, fclToken, minterRole } = await loadFixture(deployFixture);

      expect(await fclToken.hasRole(minterRole, owner.address)).to.equal(true);
    });

    it("Can be assigned", async function () {
      const { owner, account1, fclToken, minterRole } = await loadFixture(deployFixture);

      await fclToken.grantRole(minterRole, account1.address);

      expect(await fclToken.hasRole(minterRole, owner.address)).to.equal(true);
      expect(await fclToken.hasRole(minterRole, account1.address)).to.equal(true);
    });

    it("Can be unassigned", async function () {
      const { owner, account1, fclToken, minterRole } = await loadFixture(deployFixture);

      await fclToken.grantRole(minterRole, account1.address);

      expect(await fclToken.hasRole(minterRole, account1.address)).to.equal(true);

      await fclToken.revokeRole(minterRole, account1.address);

      expect(await fclToken.hasRole(minterRole, account1.address)).to.equal(false);
    });

    it("Is required to mint", async function () {
      const { owner, account1, fclToken, minterRole } = await loadFixture(deployFixture);

      const amount = 10;

      await expect(
        fclToken.mint(owner.address, amount)
      ).to.changeTokenBalance(fclToken, owner, amount);

      await expect(
        fclToken.connect(account1).mint(owner.address, amount)
      ).to.be.revertedWith(`AccessControl: account ${account1.address.toLowerCase()} is missing role ${minterRole}`);

      await fclToken.grantRole(minterRole, account1.address);

      await expect(
        fclToken.connect(account1).mint(owner.address, amount)
      ).to.changeTokenBalance(fclToken, owner, amount);
    });
  });
});
