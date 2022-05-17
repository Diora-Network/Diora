# Diora Network

https://user-images.githubusercontent.com/49777543/168764160-121147ce-78c3-4e77-a020-5e03df8e6e06.mp4

Fully EVM compatible parachain built on Substrate for Kusama and Polkadot

## Devnet Chain Specs

The Devnet is a early access live testnet used to showcase diora's products during polkadot hackathon 2022

Chain ID
```
201
```
RPC

```
https://test.diora.network/
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
