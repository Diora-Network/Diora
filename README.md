# Diora Network

https://user-images.githubusercontent.com/49777543/168764160-121147ce-78c3-4e77-a020-5e03df8e6e06.mp4

Diora Network is a incentivised smart contract parachain, utilizing advanced PoSM with Double Validation & Randomization for security guarantees

## Devnet Chain Specs

The Devnet is a public early access testnet to validate the technical architecture and security of our blockchain in a setting that is as realistic as possible.

Chain ID
```
201
```
RPC

```
https://testnet.diora.network/
```

## Run Single Development Node

To build the chain, execute the following commands from the project root:

Clone Diora
```
$ git clone 
```
Build from binary 

```
$ cargo build --release
```

To execute the chain, run:

```
$ ./target/release/diora --dev
```

The dev node also supports the use of the following flags

```
$  --dev --manual-seal
```

# Run Parachain

# Build polkadot
```
cd polkadot
cargo build --release
cd ..

# Build Diora
```
cd diora
git checkout master
cargo build --release

# Launch the multi-chain
```
polkdot-launch ./diora/polkadot-launch/config.json



