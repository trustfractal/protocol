const { expect } = require("chai");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

describe("FCLBurner", function () {
  async function deployFixture () {
    const [owner, account1] = await ethers.getSigners();

    const FCLToken = await ethers.getContractFactory("FCLToken");

    const fclToken = await FCLToken.deploy();

    const FCLBurner = await ethers.getContractFactory("FCLBurner");

    const fclBurner = await FCLBurner.deploy(fclToken.address);
    console.log(fclToken.address);

    return { owner, account1, fclToken, fclBurner };
  };

  it("Deployment should set the FCLToken contract address", async function () {
    const { fclToken, fclBurner } = await loadFixture(deployFixture);

    const fclTokenContract = await fclBurner.fclTokenContract();

    expect(fclTokenContract).to.equal(fclToken.address);
  });

  describe("Burning", function () {
    const burnId = "derp";
    const amount = 10;

    it("Should record ID and burned amount", async function () {
      const { account1, fclToken, fclBurner } = await loadFixture(deployFixture);

      await fclToken.mint(account1.address, amount);
      await fclToken.connect(account1).approve(fclBurner.address, amount);

      await fclBurner.connect(account1).burn(burnId, amount);

      const amountBurned = await fclBurner.amountBurnedById(burnId);

      expect(amountBurned).to.equal(amount);
    });

    it("Should burn", async function () {
      const { account1, fclToken, fclBurner } = await loadFixture(deployFixture);

      await fclToken.mint(account1.address, amount);
      await fclToken.connect(account1).approve(fclBurner.address, amount);

      await expect(
        fclBurner.connect(account1).burn(burnId, amount)
      ).to.changeTokenBalance(fclToken, account1, -amount);
    });

    it("Should fail if user hasn't approved FCLBurner", async function () {
      const { fclToken, fclBurner } = await loadFixture(deployFixture);
      const [_, account1] = await ethers.getSigners();

      await fclToken.mint(account1.address, amount);

      await expect(
        fclBurner.connect(account1).burn(burnId, amount)
      ).to.be.revertedWith("ERC20: insufficient allowance");

      const amountBurned = await fclBurner.amountBurnedById(burnId);
      expect(amountBurned).to.equal(0);
    });

    it("Should fail if user doesn't have enough balance", async function () {
      const { fclToken, fclBurner } = await loadFixture(deployFixture);
      const [_, account1] = await ethers.getSigners();

      await fclToken.connect(account1).approve(fclBurner.address, amount);

      await expect(
        fclBurner.connect(account1).burn(burnId, amount)
      ).to.be.revertedWith("ERC20: burn amount exceeds balance");

      expect(await fclBurner.amountBurnedById(burnId)).to.equal(0);
    });
  });
});
