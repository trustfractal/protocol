const execFile = require('child_process').execFile;
const child = execFile(__dirname + '/../../../../target/release/node', ['--dev', '--tmp'], (err, stdout, stderr) => {
  if (err) {
    throw err;
  }

  console.log(stdout);
});


console.log('$ run-test', process.argv.slice(2).join(' '));

process.env.NODE_OPTIONS = `--experimental-vm-modules${
  process.env.NODE_OPTIONS
    ? ` ${process.env.NODE_OPTIONS}`
    : ''
}`;

// eslint-disable-next-line
require('jest-cli/bin/jest');
