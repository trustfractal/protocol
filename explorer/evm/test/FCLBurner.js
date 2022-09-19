const { expect } = require("chai");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

describe("FCLBurner", function() {
  async function deployFixture() {
    const [owner, user] = await ethers.getSigners();

    const FCLToken = await ethers.getContractFactory("FCLToken");
    const fclToken = await FCLToken.connect(owner).deploy();

    const FCLBurner = await ethers.getContractFactory("FCLBurner");
    const fclBurner = await FCLBurner.connect(owner).deploy(fclToken.address);

    return { owner, user, fclToken, fclBurner };
  };

  it("deployment sets the FCLToken contract address", async function() {
    const { fclToken, fclBurner } = await loadFixture(deployFixture);

    const fclTokenContract = await fclBurner.fclTokenContract();

    expect(fclTokenContract).to.equal(fclToken.address);
  });

  describe("burning", function() {
    const burnId = "derp";
    const amount = 10;

    it("burns", async function() {
      const { user, fclToken, fclBurner } = await loadFixture(deployFixture);

      await fclToken.mint(user.address, amount);
      await fclToken.connect(user).approve(fclBurner.address, amount);

      await expect(
        fclBurner.connect(user).burn(burnId, amount)
      ).to.changeTokenBalance(fclToken, user, -amount);
    });

    it("records ID and burned amount", async function() {
      const { user, fclToken, fclBurner } = await loadFixture(deployFixture);

      await fclToken.mint(user.address, amount);
      await fclToken.connect(user).approve(fclBurner.address, amount);

      await fclBurner.connect(user).burn(burnId, amount);

      const amountBurned = await fclBurner.amountBurnedById(burnId);
      expect(amountBurned).to.equal(amount);
    });

    it("fails if user hasn't approved FCLBurner", async function() {
      const { fclToken, fclBurner } = await loadFixture(deployFixture);
      const [_, user] = await ethers.getSigners();

      await fclToken.mint(user.address, amount);

      await expect(
        fclBurner.connect(user).burn(burnId, amount)
      ).to.be.revertedWith("ERC20: insufficient allowance");

      const amountBurned = await fclBurner.amountBurnedById(burnId);
      expect(amountBurned).to.equal(0);
    });

    it("fails if user doesn't have enough balance", async function() {
      const { fclToken, fclBurner } = await loadFixture(deployFixture);
      const [_, user] = await ethers.getSigners();

      await fclToken.connect(user).approve(fclBurner.address, amount);

      await expect(
        fclBurner.connect(user).burn(burnId, amount)
      ).to.be.revertedWith("ERC20: burn amount exceeds balance");

      expect(await fclBurner.amountBurnedById(burnId)).to.equal(0);
    });
  });
});
