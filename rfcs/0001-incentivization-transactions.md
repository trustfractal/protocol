# Summary

[summary]: #summary

Provide a mechanism to pay users for having a verified identity and continuously adding data to their data host.

# Motivation

[motivation]: #motivation

The protocol is deeply dependant on the network effects of users wanting applications and applications wanting users.
To grow faster, we want to directly incentivize users to be on the network and create data.

We expect users to proactively set up their identity, data capture, and data hosting to receive FCL tokens.

# Guide explanation

[guide-explanation]: #guide-explanation

You can receive a portion of FCL minting by:

1. Hosting a signed Fractal ID message.
1. Regularly publishing the Merkle root of all data in your data host.

# Reference explanation

[reference-explanation]: #reference-explanation

Mint tokens for addresses that publish the following:

1. A unique ID signed by Fractal ID.
1. A Merkle root containing more data than the previously published one for that Fractal ID.

We will mint tokens with the following constraints:

1. Constant amount per user per day, when below total issuance per day.
   This ensures users are indifferent towards more users using the system.
1. Maximum total issued per day, with minting evenly distributed among IDs.

Additionally, communicate to users that we will retroactively be incentivizing genuine data.
Users should have these Merkle roots include their genuine browsing data, as they will be paid for that later.

# Rationale and Alternatives

[rationale-and-alternatives]: #rationale-and-alternatives

This design provides the simplest mechanism to mitigate spam (by requiring identity verification through Fractal ID).
It also incentivizes users to capture and save their data for future minting.

We could simplify this and only require a signed Fractal ID.
This limits the incentive for users to capture their data to be paid for later, as we could not verify when the data was created.

# Unresolved questions

[unresolved-questions]: #unresolved-questions

The details of how to prevent various attacks will be resolved during implementation.

- Malicious users from submitting transactions for other users.
- Malicious users from claiming multiple shares of minting.

# Future possibilities

[future-possibilities]: #future-possibilities

The next step for incentivization is to only mint for users who stake tokens on their data being genuine.
