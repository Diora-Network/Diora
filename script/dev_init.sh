#!/bin/bash
echo "building chain spec..."
./target/release/diora build-spec --chain dev --raw --disable-default-bootnode > dev-testnet.json
echo "building genesis..."
./target/release/diora export-genesis-state --chain dev > dev-genesis
echo "building wasm..."
./target/release/diora export-genesis-wasm --chain dev > dev-wasm
echo "complate!"
