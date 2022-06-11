#!/bin/bash
echo "正在生成chain spec..."
./target/release/diora build-spec --chain dev --raw --disable-default-bootnode > dev-testnet.json
echo "正在生成genesis..."
./target/release/diora export-genesis-state --chain dev > dev-genesis
echo "正在生成wasm..."
./target/release/diora export-genesis-wasm --chain dev > dev-wasm
echo "完成！"
