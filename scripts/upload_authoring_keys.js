const fs = require('fs');
const proc = require('child_process');

// Change this to the nodes/files you're uploading.
const authorities = [
  {
    ip: '3.68.37.250',
    auraKeyFile: '../keys/aura0.gpg',
    grandpaKeyFile: '../keys/gran0.gpg',
  },
  {
    ip: '3.121.242.107',
    auraKeyFile: '../keys/aura1.gpg',
    grandpaKeyFile: '../keys/gran1.gpg',
  },
  {
    ip: '3.66.246.152',
    auraKeyFile: '../keys/aura2.gpg',
    grandpaKeyFile: '../keys/gran2.gpg',
  },
];

async function main() {
  for (const auth of authorities) {
    console.log('Uploading keys to', auth.ip);
    await uploadKey(auth);
  }
}

async function uploadKey(auth) {
  const auraKey = await decryptFile(auth.auraKeyFile);
  const auraPub = await pubKey(auraKey);
  console.log('Uploading aura key', auraPub);
  await postKey(auth.ip, ["aura", auraKey, auraPub]);
  console.log('Done uploading aura key');

  const granKey = await decryptFile(auth.grandpaKeyFile);
  const granPub = await pubKey(granKey, 'Ed25519');
  console.log('Uploading gran key', granPub);
  await postKey(auth.ip, ["gran", granKey, granPub]);
  console.log('Done uploading gran key');
}

async function decryptFile(file) {
  const encrypted = await readKey(file);
  return (await decrypt(encrypted)).trim();
}

function readKey(keyFile) {
  return new Promise((resolve, reject) => {
    fs.readFile(keyFile, (err, data) => {
      if (err) return reject(err);
      resolve(data.toString());
    })
  });
}

async function decrypt(key) {
  return await runChild('gpg', ['--decrypt'], key);
}

function runChild(cmd, args = [], stdin = null) {
  return new Promise((resolve, reject) => {
    const child = proc.spawn(cmd, args);
    child.stdout.on('data', data => {
      resolve(data.toString());
    });
    if (stdin != null) {
      child.stdin.write(stdin);
      child.stdin.end();
    }
  });
}

async function pubKey(priv, scheme = 'Sr25519') {
  const inspect =
    await runChild('subkey', ['inspect', priv, '--scheme', scheme]);
  const account = inspect.match(/Account ID:\s*(0x\w+)\n/);
  return account[1];
}

async function postKey(ip, params) {
  const rpc = {
    jsonrpc: "2.0",
    id: 1,
    method: "author_insertKey",
    params,
  };
  const data = JSON.stringify(rpc);

  const options = {
    hostname: ip,
    port: 9933,
    path: '/',
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Content-Length': data.length,
    },
  };

  return new Promise((resolve, reject) => {
    const req = require('http').request(options, res => {
      if (res.statusCode !== 200)
        return reject(new Error(`Unexpected status code: ${res.statusCode}`));

      res.on('data', data => {
        resolve(data.toString());
      });
    });
    req.on('error', reject);
    req.write(data);
    req.end();
  });

}

main()
  .then(() => process.exit(0))
  .catch(err => {
    console.error(err);
    process.exit(1);
  });
