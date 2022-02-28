const fs = require("fs");
const { TxnBatcher } = require("@trustfractal/polkadot-utils");
const { Keyring, WsProvider, ApiPromise } = require("@polkadot/api");
const { decodeAddress } = require("@polkadot/keyring");
const Papa = require("papaparse");

const prompt = require("prompt");
const args = require("minimist")(process.argv.slice(2));

// Usage:
//   node send_funds.js \
//     --amounts send_funds_amounts_example.csv \
//     --out ./sent_txns.csv \
//     --network wss://nodes.testnet.fractalprotocol.com \
//     --send-at-once 256
async function main() {
  await prompt.start({ stdout: process.stderr });

  const outFile = args["out"] ?? "./sent_txns.csv";
  const amounts = parseAmounts(args.amounts, outFile, {
    allowDuplicates: args["allow-duplicates"] ?? false,
    skipInvalid: args["skip-invalid"] ?? false,
  });

  const signer = await getSigner();
  await confirmAmounts(amounts, signer);

  await sendAmounts(
    amounts,
    args.network || "wss://nodes.testnet.fractalprotocol.com",
    signer,
    { sendAtOnce: args["send-at-once"] ?? 256 },
    async (address, amount, result) => {
      const line = `${address},${Number(amount) / 10 ** 12},${result.hash}`;

      await new Promise((resolve, reject) => {
        fs.appendFile(outFile, line + "\n", (err) => {
          if (err) return reject(err);
          resolve();
        });
      });
    }
  );
}

function parseAmounts(inputPath, outFile, options) {
  const amountsFileContents = fs.readFileSync(inputPath).toString().trim();
  const {data: amountsCsv} = Papa.parse(amountsFileContents);

  // Use a map instead of an object because maps iterate their keys in insert
  // order.
  const amounts = new Map();
  let anyDuplicates = false;
  for (const [addressStr, amountStr] of amountsCsv) {
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

  const alreadySentContents = fs.readFileSync(outFile).toString().trim();
  const {data: alreadySentCsv} = Papa.parse(alreadySentContents);
  const alreadySent = new Set(alreadySentCsv.map(([address,]) => address));

  for (const addr in alreadySent) {
    amounts.delete(addr);
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

async function sendAmounts(amounts, network, signer, options, callback) {
  amounts = new Map(amounts);

  const ws = new WsProvider(network);
  const api = await ApiPromise.create({ provider: ws });
  const batcher = new TxnBatcher(api);

  while (amounts.size > 0) {
    const keys = Array.from(amounts.keys()).slice(0, options.sendAtOnce);
    const promises = keys.map(async (address) => {
      const amount = amounts.get(address);
      amounts.delete(address);

      const txn = api.tx.balances.transfer(address, amount);
      const result = await batcher.signAndSend(txn, signer).inBlock();
      await callback(address, amount, result);
    });

    await Promise.all(promises);
  }
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });
