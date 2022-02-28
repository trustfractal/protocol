const fs = require("fs");
const { TxnBatcher } = require("@trustfractal/polkadot-utils");
const { Keyring, WsProvider, ApiPromise } = require("@polkadot/api");
const { decodeAddress } = require("@polkadot/keyring");
const Papa = require("papaparse");
const cliProgress = require("cli-progress");

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
    {
      sendAtOnce: args["send-at-once"] ?? 256,
      inProgressFile:
        args["in-progress-file"] ?? "./send_funds_in_progress.txt",
    },
    async (address, amount, result) => {
      const line = `${address},${Number(amount) / 10 ** 12},${result.hash}`;
      await appendLine(outFile, line);
    }
  );
}

function parseAmounts(inputPath, outFile, options) {
  const amountsFileContents = fs.readFileSync(inputPath).toString().trim();
  const { data: amountsCsv } = Papa.parse(amountsFileContents);

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

  try {
    const alreadySentContents = fs.readFileSync(outFile).toString().trim();
    const { data: alreadySentCsv } = Papa.parse(alreadySentContents);
    const alreadySent = new Set(alreadySentCsv.map(([address]) => address));

    for (const addr of alreadySent) {
      amounts.delete(addr);
    }
  } catch (e) {
    console.error(e);
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
  const bar = new cliProgress.SingleBar({ etaBuffer: options.sendAtOnce * 4 });

  const ws = new WsProvider(network);
  const api = await ApiPromise.create({ provider: ws });
  const batcher = new TxnBatcher(api);

  bar.start(amounts.size, 0);
  try {
    while (amounts.size > 0) {
      const keys = Array.from(amounts.keys()).slice(0, options.sendAtOnce);
      const promises = keys.map(async (address) => {
        const amount = amounts.get(address);
        amounts.delete(address);

        const txn = api.tx.balances.transfer(address, amount);
        await appendLine(options.inProgressFile, txn.hash.toString());

        const result = await batcher.signAndSend(txn, signer).inBlock();
        await callback(address, amount, result);
        bar.increment();
      });

      await allSettledWithReject(promises);
    }
  } finally {
    bar.stop();
  }
}

async function allSettledWithReject(promises) {
  let oks = [];
  let errors = [];
  for (const promise of promises) {
    try {
      oks.push(await promise);
    } catch (e) {
      errors.push(e);
    }
  }
  if (errors.length > 0) {
    throw errors;
  } else {
    return oks;
  }
}

function appendLine(path, line) {
  return new Promise((resolve, reject) => {
    fs.appendFile(path, line + "\n", (err) => {
      if (err) return reject(err);
      resolve();
    });
  });
}

process.on("SIGINT", () => {
  console.warn("");
  console.warn("Got SIGINT");
  console.warn(
    "You will need to diff --in-progress-file with --out for txns that completed but the script didn't see"
  );
});

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });
