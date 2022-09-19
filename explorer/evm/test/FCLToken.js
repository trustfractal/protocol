const { expect } = require("chai");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

describe("FCLToken", function () {
  async function deployFixture () {
    const [owner, user] = await ethers.getSigners();

    const FCLToken = await ethers.getContractFactory("FCLToken");
    const fclToken = await FCLToken.deploy();

    const minterRole = await fclToken.MINTER_ROLE();

    return { owner, user, fclToken, minterRole };
  };

  describe("minting role", function () {
    it("initially belongs to deployer", async function () {
      const { owner, fclToken, minterRole } = await loadFixture(deployFixture);

      expect(await fclToken.hasRole(minterRole, owner.address)).to.equal(true);
    });

    it("can be assigned", async function () {
      const { owner, user, fclToken, minterRole } = await loadFixture(deployFixture);

      await fclToken.grantRole(minterRole, user.address);

      expect(await fclToken.hasRole(minterRole, owner.address)).to.equal(true);
      expect(await fclToken.hasRole(minterRole, user.address)).to.equal(true);
    });

    it("can be unassigned", async function () {
      const { owner, user, fclToken, minterRole } = await loadFixture(deployFixture);

      await fclToken.grantRole(minterRole, user.address);
      expect(await fclToken.hasRole(minterRole, user.address)).to.equal(true);

      await fclToken.revokeRole(minterRole, user.address);
      expect(await fclToken.hasRole(minterRole, user.address)).to.equal(false);
    });

    it("is required to mint", async function () {
      const { owner, user, fclToken, minterRole } = await loadFixture(deployFixture);

      const amount = 10;

      await expect(
        fclToken.mint(owner.address, amount)
      ).to.changeTokenBalance(fclToken, owner, amount);

      await expect(
        fclToken.connect(user).mint(owner.address, amount)
      ).to.be.revertedWith(`AccessControl: account ${user.address.toLowerCase()} is missing role ${minterRole}`);

      await fclToken.grantRole(minterRole, user.address);
      await expect(
        fclToken.connect(user).mint(owner.address, amount)
      ).to.changeTokenBalance(fclToken, owner, amount);
    });
  });
});
