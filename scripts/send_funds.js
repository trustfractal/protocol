const fs = require("fs");
const { TxnBatcher } = require("@trustfractal/polkadot-utils");
const { Keyring, WsProvider, ApiPromise } = require("@polkadot/api");
const { decodeAddress } = require("@polkadot/keyring");
const prompt = require("prompt");
const args = require("minimist")(process.argv.slice(2));

// Usage:
//   node send_funds.js \
//     --amounts send_funds_amounts_example.csv \
//     --network wss://nodes.testnet.fractalprotocol.com
async function main() {
  await prompt.start({ stdout: process.stderr });

  const amounts = parseAmounts(args.amounts, {
    allowDuplicates: args["allow-duplicates"] ?? false,
    skipInvalid: args["skip-invalid"] ?? false,
  });

  const signer = await getSigner();
  await confirmAmounts(amounts, signer);

  await sendAmounts(
    amounts,
    args.network || "wss://nodes.testnet.fractalprotocol.com",
    signer,
    (address, amount, result) => {
      console.log(`${address},${Number(amount) / 10 ** 12},${result.hash}`);
    }
  );
}

function parseAmounts(inputPath, options) {
  const amountsFileContents = fs.readFileSync(inputPath).toString().trim();

  // Use a map instead of an object because maps iterate their keys in insert
  // order.
  const amounts = new Map();
  let anyDuplicates = false;
  for (const line of amountsFileContents.split("\n")) {
    const [addressStr, amountStr] = line.split(",").map((s) => s.trim());

    try {
      decodeAddress(addressStr);
    } catch (e) {
      if (options.skipInvalid) {
        console.warn(`${e.message}`);
        continue;
      } else {
        throw e;
      }
    }

    const amount = Number(amountStr) * 10 ** 12;
    if (isNaN(amount)) {
      throw new Error(`Could not parse '${amountStr}' as amount`);
    }

    const existing = amounts.get(addressStr);
    if (existing != null) {
      console.warn(`Found duplicate address: ${addressStr}`);
      anyDuplicates = true;
    }

    const next = amount + (existing ?? 0);
    amounts.set(addressStr, next);
  }

  if (!options.allowDuplicates && anyDuplicates) {
    throw new Error("Found duplicate addresses");
  }

  return amounts;
}

async function getSigner() {
  const keyring = new Keyring({ type: "sr25519" });
  const { privateKey } = await prompt.get({
    properties: {
      privateKey: {
        hidden: true,
      },
    },
  });
  return keyring.addFromUri(privateKey || "//Alice");
}

async function confirmAmounts(amounts, signer) {
  const totalToSend = Array.from(amounts.values()).reduce(
    (acc, v) => acc + BigInt(v),
    BigInt(0)
  );
  const numAccounts = amounts.size;

  const message = `Will send ${
    Number(totalToSend) / 10 ** 12
  } to ${numAccounts} addresses from ${signer.address}.`;
  console.warn(message);

  const confirmation = await prompt.get(["continue?"]);
  if (confirmation["continue?"].toLowerCase() !== "yes") {
    throw new Error("Not continuing");
  }
}

async function sendAmounts(amounts, network, signer, callback) {
  const ws = new WsProvider(network);
  const api = await ApiPromise.create({ provider: ws });
  const batcher = new TxnBatcher(api);

  const promises = Array.from(amounts.entries()).map(
    async ([address, amount]) => {
      const txn = api.tx.balances.transfer(address, amount);
      const result = await batcher.signAndSend(txn, signer).inBlock();
      await callback(address, amount, result);
    }
  );

  await Promise.all(promises);
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });
