require("@nomicfoundation/hardhat-toolbox");

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: "0.8.17",
  networks: {
    hardhat: {
      mining: {
        auto: false,
        interval: [3000, 6000],
      },
    },
    gnosis: {
      url: "https://rpc.gnosischain.com/",
      //hardfork: "merge",
      //gasPrice: 1000000000,
      gasPrice: 10000000000,
      accounts: [process.env.PRIVATE_KEY_GNOSIS],
    },
  },
};
