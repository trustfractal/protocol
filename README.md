# Fractal Protocol

The Fractal Protocol is a open source, zero margin, protocol for decentralizing behavioral data capture and use.
Basically, this means that with the Fractal Protocol, your behavioral data (Searches, Video Views, Webpage Views)
is captured and stored by your device.
Then, when someone wants to access this data, you have to explicitly give or deny permission.

The protocol also includes a variety of mechanisms for applications to access your data.
So maybe you don't want to share your Searches directly, but you're willing to share how frequently you search.
An application could request to run an algorithm on your searches that calculates your searches per day.

## Project

This project is a monorepo to simplify coordinating changes across multiple aspects of the project.

The project is written as much as makes sense in Rust for deveeloper experience, performance, correctness, and consitency.

## Links

* [Planning - Pivotal Tracker](https://www.pivotaltracker.com/n/projects/2498241)
  * [Personas](https://docs.google.com/document/d/14HPF7GcAi75JpoHObRmxJat71pEHb7PEooXsEQZyzPY)
* [Mainnet Nodes](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fnodes.mainnet.fractalprotocol.com#/explorer)
  * wss://nodes.mainnet.fractalprotocol.com
* [Field Guide](https://fractal-id.gitbook.io/public-fractal-protocol-field-guide/)
  * Good reading for ideas behind how this could be used.

### Dev Links

* [Testnet Nodes](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fnodes.testnet.fractalprotocol.com#/explorer)
  * wss://nodes.testnet.fractalprotocol.com

## Running

### Blockchain Node (Substrate)

From the root directory:

```
# For local development
cargo run --release --bin node -- --dev --tmp
```

## Contributing

Being an open-source project, we welcome contributions.
However, this is a complex project still in early stages where correctness is critical.
That means, we will likely reject 3000 line PRs that "implement" new features.

Instead, please open an issue early on in your process so we can discuss approaches and how to integrate your change.
Then, plan to create multiple small commits that incrementally add value or work toward adding your feature.
